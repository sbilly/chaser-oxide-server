module github.com/yourusername/chaser-oxide-examples

go 1.21

require (
	google.golang.org/grpc v1.60.0
	google.golang.org/protobuf v1.32.0
)

// 使用本地 protos 包 (相对于项目根目录)
replace chaser-oxide-server/protos => ../../protos
