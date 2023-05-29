package main

import (
	"fmt"
	"os"
	"os/exec"
)

// 用来生成放在根目录下面的test程序
func main() {
	// 生成可执行程序
	args := os.Args
	args = append([]string{"./ci/grpc/call.py"}, args[1:]...)
	cmd := exec.Command("python3", args...)
	out, _ := cmd.Output()
	fmt.Println(string(out))
}
