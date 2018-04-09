/* Architecture overview
 * =====================
 *
 * We work this by creating all of our events (from UI and from websockets) and
 * putting them into a single SPSC queue. We asynchronously fire events into
 * the queue as necessary, and then listen to new events on the queue and
 * process them in order in the process function.
 */
import { Queue } from "./queue";

// EVENT type definitions - one big union of every event that can happen
const NAME = "name"; type NAME = "name";
const WAIT = "wait"; type WAIT = "wait";
const CANCEL = "cancel"; type CANCEL = "cancel";

type Event =
    | { kind: NAME, value: string }
    | { kind: WAIT }
    | { kind: CANCEL }
;

// function with the main event loop.
const process = async (queue: Queue<Event>) => {
    const name = <HTMLInputElement>document.getElementById("name")!;
    const wait = <HTMLInputElement>document.getElementById("wait");
    const cancel = <HTMLInputElement>document.getElementById("cancel");

    let waiting = false;

    // main event loop
    while (1) {
        const next = await queue.pop();
        switch (next.kind) {
            case NAME:
                const { value } = next;
                if (value.length !== 0 && !waiting) {
                    wait.disabled = false;
                } else {
                    wait.disabled = true;
                }
                break;
            case WAIT:
                name.disabled = true;
                wait.disabled = true;
                cancel.disabled = false;
                break;
            case CANCEL:
                name.disabled = false;
                wait.disabled = false;
                cancel.disabled = true;
                break;
        }
    }
}

// setup, and start the main event loop
const main = () => {
    const queue = new Queue<Event>();
    process(queue);

    const name = <HTMLInputElement>document.getElementById("name");
    name.addEventListener("input", event => queue.push({
        kind: NAME,
        value: name.value,
    }));

    const wait = <HTMLInputElement>document.getElementById("wait");
    wait.addEventListener("click", event => queue.push({ kind: WAIT }));
    const cancel = <HTMLInputElement>document.getElementById("cancel");
    cancel.addEventListener("click", event => queue.push({ kind: CANCEL }));
}

document.addEventListener("DOMContentLoaded", main);
