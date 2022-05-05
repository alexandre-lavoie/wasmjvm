import * as React from "react";
import { useState, useEffect } from "react";
import ReactInterface from "wasmjvm_interface/react";

import * as Material from "@mui/material";
import IOPanel from "./IOPanel";
import Header from "./Header";

export default function App() {
    let [running, setRunning] = useState(false);
    let [output, setOutput] = useState("");
    let [dev, setDev] = useState(false);

    useEffect(() => {
        ReactInterface.setOutput = setOutput;
        ReactInterface.setRunning = setRunning;
        ReactInterface.setDev = setDev;
        ReactInterface.loadResources();
    }, []);

    return (
        <Material.Grid direction="column" container style={{ height: "100vh" }} spacing={2} wrap="nowrap">
            {(() => {
                if (dev) {
                    return <>
                        <Material.Grid container item xs={1}>
                            <Header running={running} />
                        </Material.Grid>
                    </>;
                } else {
                    <></>;
                }
            })()}

            <Material.Grid item xs>
                <Material.Card
                    style={{
                        height: "100%", 
                        maxHeight: "85vh", 
                        overflowY: "scroll"
                    }}
                    sx={{
                        "&::-webkit-scrollbar": {
                            width: "10px"
                        },
                        "&::-webkit-scrollbar-track": {
                            background: "rgba(0, 0, 0, 0)"
                        },
                        "&::-webkit-scrollbar-thumb": {
                            background: "#FFFFFF",
                            borderRadius: 2
                        }
                    }}
                >
                    <Material.Typography style={{ whiteSpace: "pre-wrap", fontFamily: "Courier New", padding: "1em" }}>{output}</Material.Typography>
                </Material.Card>
            </Material.Grid>

            <Material.Grid item container xs={1}>
                <IOPanel running={running} />
            </Material.Grid>
        </Material.Grid>
    )
};
