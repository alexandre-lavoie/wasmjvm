import * as React from "react";

import { useEffect, useState } from "react";
import { useFilePicker } from "use-file-picker";
import ReactInterface from "wasmjvm_interface/react";

import * as Material from "@mui/material";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import FileUploadIcon from "@mui/icons-material/FileUpload";
import SendIcon from "@mui/icons-material/Send";
import SearchIcon from "@mui/icons-material/Search";

const theme = Material.createTheme({
    palette: {
        mode: "dark"
    }
});

interface IClass {
    path: string,
    cls: string,
    main: boolean
}

export default function App() {
    let [output, setOutput] = useState("");
    let [input, setInput] = useState("");
    let [running, setRunning] = useState(false);
    let [jarCount, setJarCount] = useState(0);

    ReactInterface.setOutput = setOutput;
    ReactInterface.setRunning = setRunning;

    let [openFileSelector, { plainFiles }] = useFilePicker({
        accept: "*.*"
    });

    useEffect((async () => {
        let jarCountDelta = 0;

        await Promise.all(Object.values(plainFiles).map(async (file) => {
            let buffer = await file.arrayBuffer();
            let uint8 = new Uint8Array(buffer);

            if (file.name.endsWith(".jar")) {
                ReactInterface.loadJar(uint8);
                jarCountDelta += 1;
            }
        }));

        setJarCount(jarCount + jarCountDelta);
    }) as any, [plainFiles]);

    return (
        <Material.ThemeProvider theme={theme}>
            <Material.Grid direction="column" container style={{ height: "100vh" }} spacing={2} wrap="nowrap">
                <Material.Grid container item xs={1}>
                    <Material.AppBar style={{ width: "100%", padding: "1em" }} position="static" elevation={0}>
                        <Material.Typography>WasmJVM</Material.Typography>
                    </Material.AppBar>
                </Material.Grid>

                <Material.Grid item xs={10}>
                    <Material.Card style={{ height: "100%" }} square>
                        <Material.Typography style={{ whiteSpace: "pre-wrap", fontFamily: "Courier New", padding: "1em" }}>{output}</Material.Typography>
                    </Material.Card>
                </Material.Grid>

                <Material.Grid container item spacing={2} xs={1}>
                    <Material.Grid item xs>
                        <Material.TextField style={{ width: "100%" }} value={input} onChange={(e) => setInput(e.target.value)} />
                    </Material.Grid>

                    <Material.Grid item xs={2}>
                        <Material.Card style={{ padding: "0.5em" }} square>
                            <Material.Grid container spacing={1}>
                                {
                                    (() => {
                                        if (running) {
                                            return (
                                                <Material.Grid item>
                                                    <Material.Tooltip title="Send">
                                                        <Material.IconButton onClick={() => {
                                                            ReactInterface.stdin(input);
                                                            setInput("");
                                                        }}>
                                                            <SendIcon />
                                                        </Material.IconButton>
                                                    </Material.Tooltip>
                                                </Material.Grid>
                                            );
                                        } else {
                                            return <>
                                                <Material.Grid item>
                                                    <Material.Tooltip title="Toggle Play">
                                                        <Material.IconButton onClick={() => ReactInterface.run()}>
                                                            <PlayArrowIcon />
                                                        </Material.IconButton>
                                                    </Material.Tooltip>
                                                </Material.Grid>
                                                <Material.Grid item>
                                                    <Material.Tooltip title="Upload Class/JAR">
                                                        <Material.IconButton onClick={() => openFileSelector()}>
                                                            <FileUploadIcon />
                                                        </Material.IconButton>
                                                    </Material.Tooltip>
                                                </Material.Grid>
                                            </>;
                                        }
                                    })()
                                }
                                <Material.Grid item>
                                    <Material.Tooltip title="Show Info">
                                        <Material.IconButton>
                                            <Material.Badge badgeContent={jarCount} color="primary">
                                                <SearchIcon />
                                            </Material.Badge>
                                        </Material.IconButton>
                                    </Material.Tooltip>
                                </Material.Grid>
                            </Material.Grid>
                        </Material.Card>
                    </Material.Grid>
                </Material.Grid>
            </Material.Grid>
        </Material.ThemeProvider>
    )
};
