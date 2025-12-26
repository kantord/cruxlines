export const Status = Object.freeze({
  ACTIVE: "active",
  INACTIVE: "inactive",
});

export class User {
  constructor(name, status) {
    this.name = name;
    this.status = status;
  }
}
