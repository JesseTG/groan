import { createRoot } from "react-dom/client";

import "@assets/index.css"; // So the CSS is built by the build script
import {ServiceCalls} from "./messages";

let root = document.getElementById("root");
createRoot(root!).render(<ServiceCalls />);