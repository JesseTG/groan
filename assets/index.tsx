import { useState } from "react";
import { createRoot } from "react-dom/client";
import { Button, Dialog, DialogHeading } from "@ariakit/react";

import "@assets/index.css"; // So the CSS is built by the build script

function App() {
    const [open, setOpen] = useState(false);
    return (
        <>
            <Button onClick={() => setOpen(true)}>Open dialog</Button>
            <Dialog open={open} onClose={() => setOpen(false)}>
                <DialogHeading>Ariakit</DialogHeading>
                <p>Welcome to Ariakit!</p>
            </Dialog>
        </>
    );
}

let root = document.getElementById("root");
console.log(root);
createRoot(root!).render(<App />);