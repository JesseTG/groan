import {ReactNode, useState, useEffect, ReactElement} from "react";
import {Collection, CollectionItem, Button, Dialog, DialogDismiss, DialogHeading} from "@ariakit/react";
import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then((res) => res.json());

function Details() {
    return (
        <div>
            <DialogDismiss className="button">OK</DialogDismiss>
        </div>
    )
}

function MessageButton({id, image, imageAlt, children}: { id: number, image?: string, imageAlt?: string, children: ReactNode }) {
    const [open, setOpen] = useState(false);

    let imageElement: ReactElement | null = null;
    if (image) {
        imageElement = <img src={image} alt={imageAlt} height="48"/>;
    }
    return (
        <>
            <Button className="button" onClick={() => setOpen(true)}>
                {imageElement}
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
                {children}
                <DialogDismiss className="button secondary">OK</DialogDismiss>
            </Dialog>
        </>
    );
}

export function ClientRequest({id}: { id: number }) {
    const imageUrl = `/api/request/${id}/image`;
    return (
        <MessageButton id={id} image={imageUrl} imageAlt={`Screenshot #${id}`}>
            <img src={imageUrl} alt={`Screenshot #${id}`}/>
            <dl>
                <dt>doge</dt>
                <dd>france</dd>
                <dt>wow</dt>
                <dd>much test</dd>
            </dl>
        </MessageButton>
        // TODO: Display the JSON object inline
    );
}

export function OpenAiRequest({id}: { id: number }) {
    return (
        <MessageButton id={id}>
            Request
        </MessageButton>
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
        <ClientRequest id={id} />
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