@file:JvmName("InteropKt")

fun kotlinGreet(): String {
    return "hi"
}

fun useJava(): String {
    val user = JavaUser("Ada")
    return user.getName()
}
