// @generated by protobuf-ts 2.9.4
// @generated from protobuf file "api.proto" (package "api", syntax proto3)
// tslint:disable
import { ServiceType } from "@protobuf-ts/runtime-rpc";
import type { BinaryWriteOptions } from "@protobuf-ts/runtime";
import type { IBinaryWriter } from "@protobuf-ts/runtime";
import { WireType } from "@protobuf-ts/runtime";
import type { BinaryReadOptions } from "@protobuf-ts/runtime";
import type { IBinaryReader } from "@protobuf-ts/runtime";
import { UnknownFieldHandler } from "@protobuf-ts/runtime";
import type { PartialMessage } from "@protobuf-ts/runtime";
import { reflectionMergePartial } from "@protobuf-ts/runtime";
import { MessageType } from "@protobuf-ts/runtime";
/**
 * @generated from protobuf message api.StatusResponse
 */
export interface StatusResponse {
    /**
     * @generated from protobuf field: uint32 status = 1;
     */
    status: number;
    /**
     * @generated from protobuf field: string statusText = 2;
     */
    statusText: string;
}
/**
 * @generated from protobuf message api.LogFilter
 */
export interface LogFilter {
    /**
     * @generated from protobuf field: map<string, string> fields = 1;
     */
    fields: {
        [key: string]: string;
    };
    /**
     * @generated from protobuf field: optional string namespace = 2;
     */
    namespace?: string;
}
/**
 * @generated from protobuf message api.LogMessage
 */
export interface LogMessage {
    /**
     * @generated from protobuf field: map<string, string> fields = 1;
     */
    fields: {
        [key: string]: string;
    };
    /**
     * @generated from protobuf field: string message = 2;
     */
    message: string;
    /**
     * @generated from protobuf field: optional string namespace = 3;
     */
    namespace?: string;
}
// @generated message type with reflection information, may provide speed optimized methods
class StatusResponse$Type extends MessageType<StatusResponse> {
    constructor() {
        super("api.StatusResponse", [
            { no: 1, name: "status", kind: "scalar", T: 13 /*ScalarType.UINT32*/ },
            { no: 2, name: "statusText", kind: "scalar", T: 9 /*ScalarType.STRING*/ }
        ]);
    }
    create(value?: PartialMessage<StatusResponse>): StatusResponse {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.status = 0;
        message.statusText = "";
        if (value !== undefined)
            reflectionMergePartial<StatusResponse>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: StatusResponse): StatusResponse {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* uint32 status */ 1:
                    message.status = reader.uint32();
                    break;
                case /* string statusText */ 2:
                    message.statusText = reader.string();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: StatusResponse, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* uint32 status = 1; */
        if (message.status !== 0)
            writer.tag(1, WireType.Varint).uint32(message.status);
        /* string statusText = 2; */
        if (message.statusText !== "")
            writer.tag(2, WireType.LengthDelimited).string(message.statusText);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message api.StatusResponse
 */
export const StatusResponse = new StatusResponse$Type();
// @generated message type with reflection information, may provide speed optimized methods
class LogFilter$Type extends MessageType<LogFilter> {
    constructor() {
        super("api.LogFilter", [
            { no: 1, name: "fields", kind: "map", K: 9 /*ScalarType.STRING*/, V: { kind: "scalar", T: 9 /*ScalarType.STRING*/ } },
            { no: 2, name: "namespace", kind: "scalar", opt: true, T: 9 /*ScalarType.STRING*/ }
        ]);
    }
    create(value?: PartialMessage<LogFilter>): LogFilter {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.fields = {};
        if (value !== undefined)
            reflectionMergePartial<LogFilter>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: LogFilter): LogFilter {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* map<string, string> fields */ 1:
                    this.binaryReadMap1(message.fields, reader, options);
                    break;
                case /* optional string namespace */ 2:
                    message.namespace = reader.string();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    private binaryReadMap1(map: LogFilter["fields"], reader: IBinaryReader, options: BinaryReadOptions): void {
        let len = reader.uint32(), end = reader.pos + len, key: keyof LogFilter["fields"] | undefined, val: LogFilter["fields"][any] | undefined;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case 1:
                    key = reader.string();
                    break;
                case 2:
                    val = reader.string();
                    break;
                default: throw new globalThis.Error("unknown map entry field for field api.LogFilter.fields");
            }
        }
        map[key ?? ""] = val ?? "";
    }
    internalBinaryWrite(message: LogFilter, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* map<string, string> fields = 1; */
        for (let k of globalThis.Object.keys(message.fields))
            writer.tag(1, WireType.LengthDelimited).fork().tag(1, WireType.LengthDelimited).string(k).tag(2, WireType.LengthDelimited).string(message.fields[k]).join();
        /* optional string namespace = 2; */
        if (message.namespace !== undefined)
            writer.tag(2, WireType.LengthDelimited).string(message.namespace);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message api.LogFilter
 */
export const LogFilter = new LogFilter$Type();
// @generated message type with reflection information, may provide speed optimized methods
class LogMessage$Type extends MessageType<LogMessage> {
    constructor() {
        super("api.LogMessage", [
            { no: 1, name: "fields", kind: "map", K: 9 /*ScalarType.STRING*/, V: { kind: "scalar", T: 9 /*ScalarType.STRING*/ } },
            { no: 2, name: "message", kind: "scalar", T: 9 /*ScalarType.STRING*/ },
            { no: 3, name: "namespace", kind: "scalar", opt: true, T: 9 /*ScalarType.STRING*/ }
        ]);
    }
    create(value?: PartialMessage<LogMessage>): LogMessage {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.fields = {};
        message.message = "";
        if (value !== undefined)
            reflectionMergePartial<LogMessage>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: LogMessage): LogMessage {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* map<string, string> fields */ 1:
                    this.binaryReadMap1(message.fields, reader, options);
                    break;
                case /* string message */ 2:
                    message.message = reader.string();
                    break;
                case /* optional string namespace */ 3:
                    message.namespace = reader.string();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    private binaryReadMap1(map: LogMessage["fields"], reader: IBinaryReader, options: BinaryReadOptions): void {
        let len = reader.uint32(), end = reader.pos + len, key: keyof LogMessage["fields"] | undefined, val: LogMessage["fields"][any] | undefined;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case 1:
                    key = reader.string();
                    break;
                case 2:
                    val = reader.string();
                    break;
                default: throw new globalThis.Error("unknown map entry field for field api.LogMessage.fields");
            }
        }
        map[key ?? ""] = val ?? "";
    }
    internalBinaryWrite(message: LogMessage, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* map<string, string> fields = 1; */
        for (let k of globalThis.Object.keys(message.fields))
            writer.tag(1, WireType.LengthDelimited).fork().tag(1, WireType.LengthDelimited).string(k).tag(2, WireType.LengthDelimited).string(message.fields[k]).join();
        /* string message = 2; */
        if (message.message !== "")
            writer.tag(2, WireType.LengthDelimited).string(message.message);
        /* optional string namespace = 3; */
        if (message.namespace !== undefined)
            writer.tag(3, WireType.LengthDelimited).string(message.namespace);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message api.LogMessage
 */
export const LogMessage = new LogMessage$Type();
/**
 * @generated ServiceType for protobuf service api.JournalLogStream
 */
export const JournalLogStream = new ServiceType("api.JournalLogStream", [
    { name: "Tail", serverStreaming: true, options: {}, I: LogFilter, O: LogMessage }
]);
