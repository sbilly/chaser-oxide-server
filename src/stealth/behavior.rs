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

    /// Generate Bezier curve for mouse movement
    fn generate_bezier_path(
        start: (f64, f64),
        end: (f64, f64),
        options: &MouseMoveOptions,
    ) -> Vec<(f64, f64)> {
        let mut rng = rand::thread_rng();

        // Generate control points with deviation
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        let cp1 = (
            start.0 + dx * 0.25 + (rng.gen::<f64>() - 0.5) * options.deviation,
            start.1 + dy * 0.25 + (rng.gen::<f64>() - 0.5) * options.deviation,
        );

        let cp2 = (
            end.0 - dx * 0.25 + (rng.gen::<f64>() - 0.5) * options.deviation,
            end.1 - dy * 0.25 + (rng.gen::<f64>() - 0.5) * options.deviation,
        );

        // Create cubic Bezier curve
        let bezier = Bezier::from_cubic_coordinates(
            start.0, start.1,
            cp1.0, cp1.1,
            cp2.0, cp2.1,
            end.0, end.1,
        );

        // Generate intermediate points
        let mut path = Vec::new();
        for i in 0..=options.points {
            let t = i as f64 / options.points as f64;
            let point = bezier.evaluate(bezier_rs::TValue::Euclidean(t));
            path.push((point[0], point[1]));
        }

        path
    }

    /// Calculate typing delay
    fn calculate_typing_delay(options: &TypingOptions) -> u64 {
        let mut rng = rand::thread_rng();

        // Use Gaussian distribution for realistic typing
        let delay = (rng.gen::<f64>() * 2.0 - 1.0) * options.std_dev_ms as f64
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
        // Pre-generate all random decisions to avoid holding ThreadRng across await
        let (typing_actions, _chars): (Vec<Vec<TypingAction>>, Vec<char>) = {
            let mut rng = rand::thread_rng();
            let chars_vec: Vec<char> = text.chars().collect();
            let mut actions = Vec::new();

            for ch in &chars_vec {
                let mut char_actions = Vec::new();

                // Check for typo
                if rng.gen::<f64>() < options.typo_probability {
                    let wrong_char = rng.gen_range('a'..='z');
                    char_actions.push(TypingAction::WrongChar(wrong_char));
                    char_actions.push(TypingAction::Delay(100));
                    char_actions.push(TypingAction::Backspace);
                    char_actions.push(TypingAction::Delay(200));
                }

                // Check for backspace simulation
                if rng.gen::<f64>() < options.backspace_probability {
                    char_actions.push(TypingAction::Backspace);
                    char_actions.push(TypingAction::Delay(150));
                }

                // Type actual character
                char_actions.push(TypingAction::TypeChar(*ch));

                // Realistic delay between keystrokes
                let delay = Self::calculate_typing_delay(&options);
                char_actions.push(TypingAction::Delay(delay as u32));

                actions.push(char_actions);
            }
            // rng dropped here
            (actions, chars_vec)
        };

        // Focus element first
        let focus_params = serde_json::json!({
            "objectId": element_id
        });

        self.cdp_client
            .call_method("DOM.focus", focus_params)
            .await?;

        // Execute pre-generated typing actions
        for actions in typing_actions {
            for action in actions {
                match action {
                    TypingAction::TypeChar(c) => {
                        self.type_char(page_id, c).await?;
                    }
                    TypingAction::WrongChar(c) => {
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
        // Get element position
        let params = serde_json::json!({
            "objectId": element_id
        });

        let result = self
            .cdp_client
            .call_method("DOM.getBoxModel", params)
            .await?;

        // Extract center position
        let model = result
            .get("model")
            .ok_or_else(|| Error::ScriptExecutionFailed("No box model found".to_string()))?;

        let content = model
            .get("content")
            .ok_or_else(|| Error::ScriptExecutionFailed("No content quad found".to_string()))?;

        let quad = content
            .as_array()
            .ok_or_else(|| Error::ScriptExecutionFailed("Invalid quad format".to_string()))?;

        let x = (quad[0].as_f64().unwrap_or(0.0) + quad[4].as_f64().unwrap_or(0.0)) / 2.0;
        let y = (quad[1].as_f64().unwrap_or(0.0) + quad[5].as_f64().unwrap_or(0.0)) / 2.0;

        // Delay before click
        tokio::time::sleep(Duration::from_millis(options.delay_before_ms)).await;

        // Move mouse to element
        let current_pos = (0.0, 0.0);
        let mouse_options = MouseMoveOptions {
            duration_ms: options.movement_duration_ms,
            deviation: 20.0,
            points: 10,
        };

        self.simulate_mouse_move(page_id, current_pos, (x, y), mouse_options)
            .await?;

        // Press mouse
        let press_params = serde_json::json!({
            "x": x,
            "y": y,
            "type": "mousePressed",
            "button": "left",
            "clickCount": 1
        });

        self.cdp_client
            .call_method("Input.dispatchMouseEvent", press_params)
            .await?;

        // Hold duration
        tokio::time::sleep(Duration::from_millis(options.hold_duration_ms)).await;

        // Release mouse
        let release_params = serde_json::json!({
            "x": x,
            "y": y,
            "type": "mouseReleased",
            "button": "left",
            "clickCount": 1
        });

        self.cdp_client
            .call_method("Input.dispatchMouseEvent", release_params)
            .await?;

        Ok(())
    }

    /// Simulate scroll behavior
    async fn simulate_scroll(
        &self,
        _page_id: &str,
        target_y: f64,
        options: ScrollOptions,
    ) -> Result<(), Error> {
        // Generate random delays before await
        let random_factors: Vec<f32> = {
            let mut rng = rand::thread_rng();
            (0..options.steps)
                .map(|_| rng.gen_range(0.8..1.2))
                .collect()
        };
        // rng dropped here

        // Get current scroll position
        let params = serde_json::json!({});

        let result = self
            .cdp_client
            .call_method("Page.getLayoutMetrics", params)
            .await?;

        let current_y = result
            .get("cssLayoutViewport")
            .and_then(|v| v.get("pageY"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let distance = target_y - current_y;
        let step_distance = distance / options.steps as f64;

        let duration = Duration::from_millis(options.duration_ms);
        let base_delay = duration / options.steps;

        for i in 0..options.steps {
            let progress = i as f64 / options.steps as f64;

            // Add acceleration if enabled
            let multiplier = if options.acceleration {
                // Ease-in-out
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - 2.0 * (1.0 - progress) * (1.0 - progress)
                }
            } else {
                1.0
            };

            let scroll_y = current_y + step_distance * (i as f64 + 1.0) * multiplier;

            let scroll_params = serde_json::json!({
                "x": 0.0,
                "y": scroll_y,
                "type": "mouseWheel"
            });

            self.cdp_client
                .call_method("Input.dispatchMouseEvent", scroll_params)
                .await?;

            // Add randomness to delay using pre-generated factor
            let random_delay = base_delay.mul_f32(random_factors[i as usize]);
            tokio::time::sleep(random_delay).await;
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
        let params = serde_json::json!({
            "type": "keyDown",
            "key": ch.to_string()
        });

        self.cdp_client
            .call_method("Input.dispatchKeyEvent", params)
            .await?;

        let params = serde_json::json!({
            "type": "keyUp",
            "key": ch.to_string()
        });

        self.cdp_client
            .call_method("Input.dispatchKeyEvent", params)
            .await?;

        Ok(())
    }
}
