import * as React from "react";
import * as ReactDOM from "react-dom";
import App from "./App";
import * as Material from "@mui/material";

const theme = Material.createTheme({
    palette: {
        mode: "dark"
    },
});

ReactDOM.render(
    <React.StrictMode>
        <Material.ThemeProvider theme={theme}>
            <App />
        </Material.ThemeProvider>
    </React.StrictMode>,
    document.getElementById("root")
);
