package main

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
)

// 用来生成放在根目录下面的test程序
func main() {
	args := os.Args[1:]
	totests := []string{}

	// -u参数用来更新使用的寄存器, -p参数用来决定是否使用-p,-O指令判断是否使用优化
	flags := map[string]int{"-u": 0, "-p": 0, "-O": 0}
	for _, arg := range args {
		if _, ok := flags[arg]; ok {
			flags[arg] = 1
		} else {
			totests = append(totests, arg)
		}
	}
	if flags["-u"] == 1 {
		// 如果要更新,则把compiler传递到远程
		cmd := exec.Command("docker", []string{"cp", "./target/debug/compiler", "ci:/test/data/compiler"}...)
		out, err := cmd.Output()
		fmt.Println("update", err, ":", string(out))
		args = args[1:]
	}
	fmt.Println("test :", totests)
	flagString := ""
	flagString += strconv.Itoa(flags["-u"]) + strconv.Itoa(flags["-p"]) + strconv.Itoa(flags["-O"])

	for _, arg := range totests {
		cmd := exec.Command("python3", []string{"./ci/grpc/call.py", flagString + "#" + arg}...)
		out, _ := cmd.Output()
		fmt.Println("test" + arg + ":" + string(out))
	}
}
