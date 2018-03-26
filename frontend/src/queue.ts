// single producer, single consumer queue
// blocks on read
export class Queue<T> {
    backlog: Array<T>;
    waiting: ((item: T) => void) | null;

    constructor() {
        this.backlog = [];
        this.waiting = null;
    }

    push(item: T): void {
        if (this.waiting !== null) {
            const waiting = this.waiting;
            this.waiting = null;
            waiting(item);
        } else {
            this.backlog.push(item);
        }
    }

    pop(): Promise<T> {
        if (this.backlog.length > 0) {
            return Promise.resolve(this.backlog.shift()!);
        } else {
            return new Promise<T>(resolve => {
                this.waiting = resolve;
            });
        }
    }
}
