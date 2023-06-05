package main

import (
	"fmt"
	"os"
	"os/exec"
)

// 用来生成放在根目录下面的test程序
func main() {
	args := os.Args[1:]
	if len(args) >= 1 && args[0] == "-u" {
		// 如果要更新,则把compiler传递到远程
		exec.Command("docker", []string{"cp", "./target/debug/compiler", "ci:/test/data/compiler"}...)
		cmd := exec.Command("docker", []string{"cp", "./cie", "ci:/test/data/cie"}...)
		out, err := cmd.Output()
		fmt.Println("update", err, ":", string(out))
		args = args[1:]
	}
	fmt.Println("test :", args)
	for _, arg := range args {
		cmd := exec.Command("python3", []string{"./ci/grpc/call.py", arg}...)
		out, _ := cmd.Output()
		fmt.Println("test" + arg + ":" + string(out))
	}
}
