package main

type User struct {
	Name string
	Age  int
}

func NewUser(name string) *User {
	return &User{Name: name}
}

const DefaultAge = 18

var GlobalCounter int
