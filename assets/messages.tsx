import {ReactNode, useState, useEffect} from "react";
import {Collection, CollectionItem, Button, Dialog, DialogHeading} from "@ariakit/react";
import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then((res) => res.json());

function MessageButton({children}: { children: ReactNode }) {
    const [open, setOpen] = useState(false);
    return (
        <>
            <Button onClick={() => setOpen(true)}>{children}</Button>
            <Dialog open={open} onClose={() => setOpen(false)}>
                <DialogHeading>Message</DialogHeading>
                <p>Message</p>
            </Dialog>
        </>
    );
}

export function ClientRequest({id}: { id: number }) {
    return (
        <MessageButton>
            Request
        </MessageButton>
        // TODO: Display the image inline
        // TODO: Display the JSON object inline
    );
}

export function OpenAiRequest({id}: { id: number }) {
    return (
        <Button>
            Request
        </Button>
    );
}

export function OpenAiResponse({id}: { id: number }) {
    return (
        <Button>
            Request
        </Button>
    );
    // TODO: Display the image inline
}

export function ClientResponse({id}: { id: number }) {
    return (
        <Button>
            Response
        </Button>
    );
}

export function ServiceCall({id}: { id: number }) {
    return (<>
        <ClientRequest id={id}>
        </ClientRequest>
        <OpenAiRequest id={id}>
        </OpenAiRequest>
        <OpenAiResponse id={id}>
        </OpenAiResponse>
        <ClientResponse id={id}>
        </ClientResponse>
    </>);
    // TODO: Display the image inline
}

type RequestIds = { ids: Array<number>; };
type ServiceCallsState = { data: RequestIds | undefined, error: any, isLoading: boolean };

export function ServiceCalls() {
    const {data, error, isLoading}: ServiceCallsState = useSWR('/api/request', fetcher);

    if (isLoading) {
        return <div>Loading...</div>;
        // TODO: Make a nice-looking loading message
    }

    if (error) {
        return <div>Error: {error}</div>;
        // TODO: Make a nice-looking error message
    }

    const calls = data?.ids.map((id: number) => (
        <CollectionItem key={id}>
            <ServiceCall id={id}/>
        </CollectionItem>
    ));

    return (
        <Collection>
            {calls}
        </Collection>
    )
}