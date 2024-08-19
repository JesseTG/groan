import useSWR from "swr";
import {Button, Dialog, DialogDismiss, DialogHeading, HeadingLevel, Heading} from "@ariakit/react";
import {ReactElement, useState} from "react";
import ChatCompletionCreateParams from "openai";
import ChatCompletion from "openai";

const fetcher = (url: string) => fetch(url).then((res) => res.json());
type Headers = { [key: string]: string };
type ServiceRequest = {
    headers: Headers,
    params: string,
    body: RequestBody,
}

type ServiceResponse = {
    headers: Headers,
    body: ResponseBody,
}

type InputButtonState = 0 | 1;
type InputState = {
    paused: InputButtonState,
    a: InputButtonState,
    b: InputButtonState,
    x: InputButtonState,
    y: InputButtonState,
    l: InputButtonState,
    r: InputButtonState,
    l2: InputButtonState,
    r2: InputButtonState,
    l3: InputButtonState,
    r3: InputButtonState,
    up: InputButtonState,
    down: InputButtonState,
    left: InputButtonState,
    right: InputButtonState,
    select: InputButtonState,
    start: InputButtonState,
}

type RequestBody = {
    image?: string,
    format?: "png" | "bmp",
    coords?: [x: number, y: number, width: number, height: number],
    viewport?: [width: number, height: number],
    label?: string,
    state?: InputState,
};

type InputPress = 'a' | 'b' | 'x' | 'y' | 'l' | 'r' | 'l2' | 'r2' | 'l3' | 'r3' | 'up' | 'down' | 'left' | 'right' | 'select' | 'start' | 'pause' | 'unpause';
type ResponseBody = {
    image?: string,
    sound?: string,
    text?: string,
    text_position?: 1 | 2,
    press?: InputPress[],
    error?: string,
    auto?: "auto" | "continue",
}

type ServiceCallArgs = { id: number };
type ServiceCallState = { data?: ServiceCallRecord, error: any, isLoading: boolean };
type ServiceCallRecord = {
    id: number,
    client_request: ServiceRequest,
    openai_request?: {CreateChatCompletionRequest: ChatCompletionCreateParams}, // OpenAiRequest,
    openai_response?: {CreateChatCompletionResponse: ChatCompletion}, //OpenAiResponse
    client_response?: ServiceResponse, // ServiceResponse,
};

function ClientRequest({request}: { request: ServiceRequest }) {
    function* headerElements(headers: object) {
        for (const [key, value] of Object.entries(headers)) {
            yield (<>
                <dt key={key}><code>{key}</code></dt>
                <dd key={value}><code>{value}</code></dd>
            </>); // TODO: Handle the case where key = value
        }
    }

    return (
        <HeadingLevel>
            <Heading>Client Request</Heading>
            <HeadingLevel>
                <Heading>Query Parameters</Heading>
                <pre>{request.params}</pre>
            </HeadingLevel>
            <HeadingLevel>
                <Heading>Request Headers</Heading>
                <dl>
                    {[...headerElements(request.headers!)]}
                </dl>
            </HeadingLevel>
            <HeadingLevel>
                <Heading>Request Body</Heading>
                <dl>
                    <dt>Format</dt>
                    <dd>{request.body.format}</dd>
                </dl>
            </HeadingLevel>
        </HeadingLevel>
    );
}

function OpenAiRequest({request}: { request?: ChatCompletionCreateParams }) {
    if (!request) {
        return <></>;
    }

    return (
        <HeadingLevel>
            <Heading>OpenAI Request</Heading>
            <pre>{JSON.stringify(request, null, 2)}</pre>
        </HeadingLevel>
    );
}

function OpenAiResponse({response}: { response?: ChatCompletion }) {
    if (!response) {
        return <></>;
    }

    return (
        <HeadingLevel>
            <Heading>OpenAI Response</Heading>
            <pre>{JSON.stringify(response, null, 2)}</pre>
        </HeadingLevel>
    );
}

function ClientResponse({response}: { response: ServiceResponse | undefined }) {
    if (!response) {
        return <></>;
    }

    return (
        <HeadingLevel>
            <Heading>Client Response</Heading>
            <pre>{JSON.stringify(response, null, 2)}</pre>
        </HeadingLevel>
    );
}


export default function ServiceCall({id}: ServiceCallArgs) {
    const [open, setOpen] = useState(false);
    const {data, error, isLoading}: ServiceCallState = useSWR(`/api/request/${id}`, fetcher);
    const imageUrl = `/api/request/${id}/image`;

    if (isLoading) {
        return <div>Loading...</div>;
    } // TODO: Make a nice-looking loading message

    if (error) {
        return <div>Error: {error}</div>;
    } // TODO: Make a nice-looking error message


    return (<>
        <Button className="button" onClick={() => setOpen(true)}>
            <img src={imageUrl} alt={`Screenshot #${id}`} height="48"/>
        </Button>
        <Dialog
            open={open}
            backdrop={false}
            className="dialog"
            onClose={() => setOpen(false)}
            render={(props) => (
                <div className="backdrop" hidden={!open}>
                    <div {...props} />
                </div>
            )}
        >
            <DialogHeading className="heading">Request #{id}</DialogHeading>
            <img src={imageUrl} alt={`Screenshot #${id}`}/>
            <div>
                <ClientRequest request={data!.client_request}/>
                <OpenAiRequest request={data?.openai_request?.CreateChatCompletionRequest}/>
                <OpenAiResponse response={data!.openai_response?.CreateChatCompletionResponse}/>
                <ClientResponse response={data!.client_response}/>
            </div>


            <DialogDismiss className="button secondary">OK</DialogDismiss>
        </Dialog>
    </>);
}