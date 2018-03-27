# Class Queue
This is a web-based tool for tracking people that need help in a classroom environment.
Allows users to enter their name and join the queue, and then withdraw their position from the queue at a later time.

# Tech Stack
The front end and back end communicate purely through websockets, so that updates are automatic.

The front end consists of simple html / sass / typescript.
No frameworks for rendering or creating layouts dynamically from javascript are currently used.

The back end is written in rust, and consists of a single threaded, asynchronous server serving over TLS.
Currently the static files will need an extra server to host them, but this is going to be fixed in future.
Note that rust is not necessarily the best tool for this job, its use here is experimental.

# Building
The front end uses yarn and parcel js to build.
A development server can be started with `yarn parcel index.html`.
A distributable set of static files can be generated with `yarn parcel build index.html`.

The back end uses cargo to build.
A development build can be run with `cargo run`.
A production binary can be created with `cargo build --release`.
