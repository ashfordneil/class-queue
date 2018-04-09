export type ClientMessage =
    | { type: "Await", content: string }
    | { type: "Cancel", content: string }
    | { type: "Authenticate", content: string }
    | { type: "Visit", content: string }
;

export type ServerMessage =
    | { type: "YourId", content: string }
    | { type: "UnAuthorized" }
    | { type: "NewQueue", content: [Student] }
;

export interface Student {
    being_seen: boolean;
    name: string;
}
