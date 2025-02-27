// import './components/logstream'
import {GrpcWebFetchTransport} from "@protobuf-ts/grpcweb-transport";
import './grpc-journal/api.client'
import {JournalLogStreamClient} from "./grpc-journal/api.client";

let transport = new GrpcWebFetchTransport({baseUrl: ""});
let client = new JournalLogStreamClient(transport);
let stream = client.tail({namespace: "ncatomic", fields: {"_NAMESPACE": "ncatomic"}})
stream.responses.onNext((message, error, complete) => {
    // @ts-ignore
    dioxus.send(JSON.stringify({message, error, complete}));
})
