//! Behavior simulator implementation
//!
//! Simulates human-like behavior patterns to evade detection.

use std::time::Duration;
use std::sync::Arc;
use async_trait::async_trait;
use rand::Rng;
use bezier_rs::Bezier;

use crate::{Error, cdp::CdpClient};
use super::traits::*;

/// Typing action for pre-generated typing sequences
enum TypingAction {
    TypeChar(char),
    WrongChar(char),
    Backspace,
    Delay(u32),
}

/// Behavior simulator implementation
pub struct BehaviorSimulatorImpl {
    /// CDP client
    cdp_client: Arc<dyn CdpClient>,
}

impl BehaviorSimulatorImpl {
    /// Create a new behavior simulator
    pub fn new(cdp_client: Arc<dyn CdpClient>) -> Self {
        Self { cdp_client }
    }

    /// Generate Bezier curve path for mouse movement
    fn generate_bezier_path(
        start: (f64, f64),
        end: (f64, f64),
        options: &MouseMoveOptions,
    ) -> Vec<(f64, f64)> {
        let (dx, dy) = (end.0 - start.0, end.1 - start.1);
        let deviation = options.deviation;

        // Generate control points with random deviation
        let cp1 = (
            start.0 + dx * 0.25 + (rand::random::<f64>() - 0.5) * deviation,
            start.1 + dy * 0.25 + (rand::random::<f64>() - 0.5) * deviation,
        );

        let cp2 = (
            end.0 - dx * 0.25 + (rand::random::<f64>() - 0.5) * deviation,
            end.1 - dy * 0.25 + (rand::random::<f64>() - 0.5) * deviation,
        );

        let bezier = Bezier::from_cubic_coordinates(
            start.0, start.1,
            cp1.0, cp1.1,
            cp2.0, cp2.1,
            end.0, end.1,
        );

        (0..=options.points)
            .map(|i| {
                let t = i as f64 / options.points as f64;
                let point = bezier.evaluate(bezier_rs::TValue::Euclidean(t));
                (point[0], point[1])
            })
            .collect()
    }

    /// Calculate typing delay with Gaussian distribution
    fn calculate_typing_delay(options: &TypingOptions) -> u64 {
        let delay = (rand::random::<f64>() * 2.0 - 1.0) * options.std_dev_ms as f64
            + options.mean_delay_ms as f64;
        delay.max(10.0) as u64
    }
}

#[async_trait]
impl BehaviorSimulator for BehaviorSimulatorImpl {
    /// Simulate mouse movement using Bezier curves
    async fn simulate_mouse_move(
        &self,
        _page_id: &str,
        start: (f64, f64),
        end: (f64, f64),
        options: MouseMoveOptions,
    ) -> Result<(), Error> {
        let path = Self::generate_bezier_path(start, end, &options);

        let duration = Duration::from_millis(options.duration_ms);
        let step_delay = duration / path.len() as u32;

        for (x, y) in path {
            let params = serde_json::json!({
                "x": x,
                "y": y,
                "type": "mouseMoved"
            });

            self.cdp_client
                .call_method("Input.dispatchMouseEvent", params)
                .await?;

            tokio::time::sleep(step_delay).await;
        }

        Ok(())
    }

    /// Simulate human-like typing
    async fn simulate_typing(
        &self,
        page_id: &str,
        element_id: &str,
        text: &str,
        options: TypingOptions,
    ) -> Result<(), Error> {
        // Focus element
        let focus_params = serde_json::json!({ "objectId": element_id });
        self.cdp_client.call_method("DOM.focus", focus_params).await?;

        // Pre-generate typing actions to avoid Send issues
        let typing_actions: Vec<Vec<TypingAction>> = text
            .chars()
            .map(|ch| {
                let mut actions = Vec::new();

                // Simulate typo
                if rand::random::<f64>() < options.typo_probability {
                    actions.push(TypingAction::WrongChar(rand::random::<char>()));
                    actions.push(TypingAction::Delay(100));
                    actions.push(TypingAction::Backspace);
                    actions.push(TypingAction::Delay(200));
                }

                // Simulate accidental backspace
                if rand::random::<f64>() < options.backspace_probability {
                    actions.push(TypingAction::Backspace);
                    actions.push(TypingAction::Delay(150));
                }

                actions.push(TypingAction::TypeChar(ch));
                actions.push(TypingAction::Delay(Self::calculate_typing_delay(&options) as u32));
                actions
            })
            .collect();

        // Execute actions
        for actions in typing_actions {
            for action in actions {
                match action {
                    TypingAction::TypeChar(c) | TypingAction::WrongChar(c) => {
                        self.type_char(page_id, c).await?;
                    }
                    TypingAction::Backspace => {
                        self.type_char(page_id, '\u{08}').await?;
                    }
                    TypingAction::Delay(ms) => {
                        tokio::time::sleep(Duration::from_millis(ms as u64)).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Simulate human-like clicking
    async fn simulate_click(
        &self,
        page_id: &str,
        element_id: &str,
        options: ClickOptions,
    ) -> Result<(), Error> {
        // Get and extract center position
        let result = self.cdp_client.call_method("DOM.getBoxModel", serde_json::json!({ "objectId": element_id })).await?;
        let quad = result.get("model")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
            .ok_or_else(|| Error::ScriptExecutionFailed("Invalid box model".to_string()))?;

        let x = (quad[0].as_f64().unwrap_or(0.0) + quad[4].as_f64().unwrap_or(0.0)) / 2.0;
        let y = (quad[1].as_f64().unwrap_or(0.0) + quad[5].as_f64().unwrap_or(0.0)) / 2.0;

        // Move, press, hold, release
        tokio::time::sleep(Duration::from_millis(options.delay_before_ms)).await;
        self.simulate_mouse_move(page_id, (0.0, 0.0), (x, y), MouseMoveOptions {
            duration_ms: options.movement_duration_ms,
            deviation: 20.0,
            points: 10,
        }).await?;

        for (event_type, delay) in [("mousePressed", options.hold_duration_ms), ("mouseReleased", 0)] {
            self.cdp_client.call_method("Input.dispatchMouseEvent", serde_json::json!({
                "x": x, "y": y, "type": event_type, "button": "left", "clickCount": 1
            })).await?;
            if delay > 0 { tokio::time::sleep(Duration::from_millis(delay)).await; }
        }

        Ok(())
    }

    /// Simulate scroll behavior
    async fn simulate_scroll(
        &self,
        _page_id: &str,
        target_y: f64,
        options: ScrollOptions,
    ) -> Result<(), Error> {
        // Generate random factors before await
        let random_factors: Vec<f32> = (0..options.steps)
            .map(|_| rand::random::<f32>() * 0.4 + 0.8)
            .collect();

        // Get current scroll position
        let result = self.cdp_client.call_method("Page.getLayoutMetrics", serde_json::json!({})).await?;
        let current_y = result.get("cssLayoutViewport")
            .and_then(|v| v.get("pageY"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let distance = target_y - current_y;
        let step_distance = distance / options.steps as f64;
        let base_delay = Duration::from_millis(options.duration_ms) / options.steps;

        for i in 0..options.steps {
            let progress = i as f64 / options.steps as f64;
            let multiplier = if options.acceleration {
                // Ease-in-out curve
                if progress < 0.5 { 2.0 * progress * progress }
                else { 1.0 - 2.0 * (1.0 - progress).powi(2) }
            } else {
                1.0
            };

            let scroll_y = current_y + step_distance * (i as f64 + 1.0) * multiplier;
            self.cdp_client.call_method("Input.dispatchMouseEvent", serde_json::json!({
                "x": 0.0, "y": scroll_y, "type": "mouseWheel"
            })).await?;

            tokio::time::sleep(base_delay.mul_f32(random_factors[i as usize])).await;
        }

        Ok(())
    }

    /// Add random delay
    async fn random_delay(&self, min_ms: u64, max_ms: u64) -> Result<(), Error> {
        // Generate random value before await to avoid Send issue
        let delay = rand::thread_rng().gen_range(min_ms..=max_ms);
        tokio::time::sleep(Duration::from_millis(delay)).await;
        Ok(())
    }
}

impl BehaviorSimulatorImpl {
    /// Type a single character
    async fn type_char(&self, _page_id: &str, ch: char) -> Result<(), Error> {
        for event_type in ["keyDown", "keyUp"] {
            self.cdp_client.call_method("Input.dispatchKeyEvent", serde_json::json!({
                "type": event_type, "key": ch.to_string()
            })).await?;
        }
        Ok(())
    }
}
