import { add, Counter, PI } from "./utils.js";
import { User, Status } from "./models.js";

function greet(name) {
  return `Hello, ${name}`;
}

const user = new User("Ada", Status.ACTIVE);
const total = add(2, 3);
const counter = new Counter(1);
counter.inc();

console.log(greet(user.name));
console.log(`total=${total}, pi=${PI}, counter=${counter.value}`);
