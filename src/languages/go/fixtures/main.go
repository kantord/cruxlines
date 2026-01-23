package main

import "fmt"

func main() {
	result := Add(1, 2)
	fmt.Println(result)

	var user User = *NewUser("Alice")
	fmt.Println(user.Name)
}
