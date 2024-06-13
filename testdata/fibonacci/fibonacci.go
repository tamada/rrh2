package main

import (
	"fmt"
	"os"
	"strconv"
)

func main() {
	var max = 15
	if len(os.Args) > 1 {
		max, _ = strconv.Atoi(os.Args[1])
	}

	var first, second = 1, 1
	for i := 1; i <= max; i++ {
		if i <= 2 {
			if i == 2 {
				fmt.Print(", ")
			}
			fmt.Printf("1")
		} else {
			var current = first + second
			first, second = second, current
			fmt.Printf(", %d", current)
		}
	}
	fmt.Println()
}
