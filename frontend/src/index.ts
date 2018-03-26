import { Queue } from "./queue";

const NAME = "name";
const WAIT = "wait";
const CANCEL = "cancel";
type NAME = "name";
type WAIT = "wait";
type CANCEL = "cancel";

type Event =
    | { kind: NAME, value: string }
    | { kind: WAIT }
    | { kind: CANCEL }
;

const main = () => {
    const queue = new Queue<Event>();
    process(queue);

    const name = document.getElementById("name");
    name.addEventListener("input", event => queue.push({
        kind: NAME,
        value: name.value,
    }));

    const wait = document.getElementById("wait");
    wait.addEventListener("click", event => queue.push({ kind: WAIT }));
    const cancel = document.getElementById("cancel");
    cancel.addEventListener("click", event => queue.push({ kind: CANCEL }));
}

const process = async (queue: Queue<Event>) => {
    const name = document.getElementById("name");
    const wait = document.getElementById("wait");
    const cancel = document.getElementById("cancel");

    let waiting = false;

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

document.addEventListener("DOMContentLoaded", main);
