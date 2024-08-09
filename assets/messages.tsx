import {Collection, CollectionItem, Button, Dialog, DialogDismiss, DialogHeading} from "@ariakit/react";
import useSWR from 'swr';
import ServiceCall from "./ServiceCall";

const fetcher = (url: string) => fetch(url).then((res) => res.json());

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