import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import {JournalLogStreamClient} from "../grpc-journal/api.client";
import {LogFilter, LogMessage} from "../grpc-journal/api";


const containerNameRe = /^.*_([^_]*)_.*$/
class Logstream extends HTMLElement {

    private uuid: string;
    private messages: LogMessage[] = [];
    private error: string | undefined = undefined;
    private filter: {"_SYSTEMD_UNIT": string[], "CONTAINER_NAME": string[]} = {"_SYSTEMD_UNIT": [], "CONTAINER_NAME": []};
    private availableFilters: {"_SYSTEMD_UNIT": string[], "CONTAINER_NAME": string[]} = {"_SYSTEMD_UNIT": [], "CONTAINER_NAME": []};
    private messageList: Element | undefined;
    private filterBar: Element | undefined;

    connectedCallback() {
        this.render();
        try {
            this.uuid = crypto.randomUUID();
        } catch (e) {
            this.uuid = 'invalid-uuid';
        }
        this.dataset.uuid = this.uuid
        let grpcAddress = this.dataset.grpcAddress
        let transport = new GrpcWebFetchTransport({baseUrl: grpcAddress});
        let client = new JournalLogStreamClient(transport);
        let stream = client.tail({namespace: "ncatomic", fields: {"_NAMESPACE": "ncatomic"}})
        stream.responses.onError((e) => {
            console.error(e)
            this.error = `${e}`;
        })
        stream.responses.onMessage((msg) => {
            console.log(msg.fields)
            if (msg.fields['PODMAN_EVENT'] !== undefined) {
                return;
            }
            this.addMessage(msg);
        })

        console.log("logstream aa connected");
    }

    addMessage(msg: LogMessage) {
        let unit = msg.fields["_SYSTEMD_UNIT"];
        let container = msg.fields["CONTAINER_NAME"];
        let rerenderRequired = false;
        if (unit !== undefined && this.availableFilters["_SYSTEMD_UNIT"].indexOf(unit) === -1) {
            this.availableFilters["_SYSTEMD_UNIT"].push(unit);
            rerenderRequired = true;
        }
        if (container !== undefined && this.availableFilters["CONTAINER_NAME"].indexOf(container) === -1) {
            this.availableFilters["CONTAINER_NAME"].push(container);
            rerenderRequired = true;
        }
        if (rerenderRequired) {
            this.rerenderFilterBar()
        }
        this.renderIncrementally(msg);
        this.messages.push(msg);
    }

    render() {
        this.innerHTML = '';
        this.filterBar = document.createElement('div');
        this.filterBar.classList.add("filters");
        this.appendChild(this.filterBar);
        this.messageList = document.createElement('div');
        this.messageList.classList.add("logstream-container", "mockup-code", "w-full", "h-96", "min-h-20vh",
            "max-h-50vh", "block", "overflow-y-scroll");
        this.messageList.innerHTML = '<li>logstream connected</li>';
        this.appendChild(this.messageList);
        this.rerenderFilterBar();
        this.rerenderMessages();
    }

    rerenderFilterBar() {
        let filterElems = this.availableFilters["_SYSTEMD_UNIT"].map((svc) => {
            let active = this.filter["_SYSTEMD_UNIT"][svc] !== undefined;
            let filterElem = document.createElement('div');
            filterElem.classList.add("filter", "service");
            if (active) {
                filterElem.classList.add('active');
            }
            filterElem.innerText = `svc:${svc}`;
            filterElem.addEventListener('click', this.toggleFilter.bind(this, "_SYSTEMD_UNIT", svc))
            return filterElem;
        })
        filterElems.push(...this.availableFilters["CONTAINER_NAME"].map((container) => {
            let active = this.filter["CONTAINER_NAME"][container] !== undefined;
            let filterElem = document.createElement('div');
            filterElem.classList.add("filter", "container");
            if (active) {
                filterElem.classList.add('active');
            }
            filterElem.innerText = `container:${this.parseContainerName(container)}`;
            filterElem.addEventListener('click', this.toggleFilter.bind(this, "CONTAINER_NAME", container));
            return filterElem;
        }))
        this.filterBar.replaceChildren(...filterElems);
    }

    toggleFilter(filterType: "_SYSTEMD_UNIT" | "CONTAINER_NAME", filterValue: string) {
        let index = this.filter[filterType].indexOf(filterValue)
        if (index === -1) {
            this.filter[filterType].push(filterValue);
        } else {
            this.filter[filterType].splice(index, 1);
        }
        this.rerenderFilterBar();
        this.rerenderMessages();
    }

    rerenderMessages() {
        let messages = this.messages
            .filter(this.matchesFilter.bind(this))
            .map(this.createMessageItem.bind(this));
        this.messageList.replaceChildren(...messages);
    }

    renderIncrementally(msg: LogMessage) {
        if (!this.matchesFilter(msg)) {
            return;
        }
        let elem = this.createMessageItem(msg, this.messageList.children.length);
        this.messageList.appendChild(elem);
    }

    matchesFilter(msg: LogMessage): boolean {
        let unit = msg.fields["_SYSTEMD_UNIT"];
        let container = msg.fields["CONTAINER_NAME"];
        if (this.filter["_SYSTEMD_UNIT"].length != 0 && this.filter["_SYSTEMD_UNIT"].indexOf(unit) === -1) {
            return false;
        }
        if (this.filter["CONTAINER_NAME"].length != 0 && this.filter["CONTAINER_NAME"].indexOf(container) === -1) {
            return false;
        }
        return true;
    }

    createMessageItem(msg: LogMessage, num: number = 0, is_fatal = false) {
        let elem = document.createElement("pre");
        elem.dataset.prefix = `${num+1}`;
        if (is_fatal) {
            elem.classList.add("bg-error", "text-error-content")
        }
        let content = `[${msg.fields['_SYSTEMD_UNIT']}]::`;
        if (msg.fields['CONTAINER_NAME'] !== undefined) {
            let container_name = this.parseContainerName(msg.fields['CONTAINER_NAME']);
            content += `<b>${container_name}</b> :: `
        }
        elem.innerHTML = "<code>" + content + `${msg.message}</code>`;
        return elem;
    }

    parseContainerName(container: string) {
        let match = containerNameRe.exec(container);
        if (match) {
            container = match[1];
        }
        if (container.indexOf("nextcloud-aio-") === 0) {
            container = container.replace("nextcloud-aio-", "");
        }
        return container;
    }
}

customElements.define('log-stream', Logstream);
