import { add } from "./utils";
import { User } from "./models";

const user = new User("Ada");
console.log(add(1, 2), user.name);
