// import './components/logstream'
import {GrpcWebFetchTransport} from "@protobuf-ts/grpcweb-transport";
import './grpc-journal/api.client'
import {JournalLogStreamClient} from "./grpc-journal/api.client";

let transport = new GrpcWebFetchTransport({baseUrl: ""});
let client = new JournalLogStreamClient(transport);
let stream = client.tail({namespace: "ncatomic", fields: {"_NAMESPACE": "ncatomic"}})
stream.responses.onMessage((message) => {
    console.log("(js)", message);
    dioxus.send(JSON.stringify({message}));
});
stream.responses.onError((error) => {
    console.error("(js)", error);
    dioxus.send(JSON.stringify({error}));
});
