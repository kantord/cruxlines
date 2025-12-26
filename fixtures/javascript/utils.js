export const PI = 3.14159;

export function add(a, b) {
  return a + b;
}

export class Counter {
  constructor(start = 0) {
    this.value = start;
  }

  inc() {
    this.value += 1;
    return this.value;
  }
}
