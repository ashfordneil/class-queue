/* Architecture overview
 * =====================
 *
 * We work this by creating all of our events (from UI and from websockets) and
 * putting them into a single SPSC queue. We asynchronously fire events into
 * the queue as necessary, and then listen to new events on the queue and
 * process them in order in the process function.
 */
import { Queue } from "./queue";
import { ClientMessage, ServerMessage } from "./protocol";

// EVENT type definitions - one big union of every event that can happen
const NAME = "name"; type NAME = "name";
const WAIT = "wait"; type WAIT = "wait";
const CANCEL = "cancel"; type CANCEL = "cancel";
const WEBSOCKET = "websocket"; type WEBSOCKET = "websocket";

type Event =
    | { kind: NAME, value: string }
    | { kind: WAIT }
    | { kind: CANCEL }
    | { kind: WEBSOCKET, value: ServerMessage }
;

// function with the main event loop.
const process = async (queue: Queue<Event>, send: (input: ClientMessage) => void) => {
    const nameField = <HTMLInputElement>document.getElementById("name")!;
    const wait = <HTMLInputElement>document.getElementById("wait");
    const cancel = <HTMLInputElement>document.getElementById("cancel");

    // What is the user's current name - provided by html form
    let name: string | null = null;
    // What is the user's current ID - provided by websocket
    let id: string | null = null;
    // Handling of race condition where user cancels before they get their ID
    // immediately cancel when you get an ID
    let cancelOnId = false;

    // main event loop
    while (1) {
        const next = await queue.pop();
        console.log(next);
        switch (next.kind) {
            case NAME:
                name = next.value || null;
                if (name !== null) {
                    wait.disabled = false;
                } else {
                    wait.disabled = true;
                }
                break;
            case WAIT:
                nameField.disabled = true;
                wait.disabled = true;
                cancel.disabled = false;

                send({ type: "Await", content: name! });
                break;
            case CANCEL:
                nameField.disabled = false;
                wait.disabled = false;
                cancel.disabled = true;

                if (id !== null) {
                    send({ type: "Cancel", content: id });
                } else {
                    cancelOnId = true;
                }
                break;
            case WEBSOCKET:
                switch (next.value.type) {
                    case "YourId":
                        if (cancelOnId) {
                            cancelOnId = false;
                        } else {
                            id = next.value.content;
                        }
                        break;
                    case "UnAuthorized":
                        break;
                    case "NewQueue":
                        break;
                }
                break;
        }
    }
}

// setup, and start the main event loop
const main = () => {
    const name = <HTMLInputElement>document.getElementById("name");
    name.addEventListener("input", event => queue.push({
        kind: NAME,
        value: name.value,
    }));

    const wait = <HTMLInputElement>document.getElementById("wait");
    wait.addEventListener("click", event => queue.push({ kind: WAIT }));
    const cancel = <HTMLInputElement>document.getElementById("cancel");
    cancel.addEventListener("click", event => queue.push({ kind: CANCEL }));

    const ws = new WebSocket("ws://localhost:8080/");
    ws.onmessage = (event) => queue.push({
        kind: WEBSOCKET,
        value: JSON.parse(event.data),
    });
    const wsSend = (input: ClientMessage) => ws.send(JSON.stringify(input));

    const queue = new Queue<Event>();
    process(queue, wsSend);
}

document.addEventListener("DOMContentLoaded", main);
