import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "./App";
import { TooltipProvider } from "@/components/ui/tooltip";
import "./index.css";

const container = document.getElementById("root");
if (!container) {
  throw new Error("#root container missing from index.html");
}

// Selah is dark-first: the operator UI runs in dimly lit auditoriums.
document.documentElement.classList.add("dark");

createRoot(container).render(
  <StrictMode>
    <TooltipProvider>
      <App />
    </TooltipProvider>
  </StrictMode>,
);
