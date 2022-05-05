import * as React from "react";
import { useEffect, useState } from "react";
import * as Material from "@mui/material";

import ReactInterface from "wasmjvm_interface/react";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import SendIcon from "@mui/icons-material/Send";

export interface IOPanel {
    running: boolean
}

const IOPanel: React.FC<IOPanel> = ({ running }) => {
    let [input, setInput] = useState("");

    return (
        <Material.Grid container spacing={2}>
            <Material.Grid item xs={11}>
                <Material.TextField style={{ width: "100%" }} value={input} onChange={(e) => setInput(e.target.value)} />
            </Material.Grid>

            <Material.Grid item xs={1}>
                <Material.Card style={{ padding: "0.5em" }}>
                    <Material.Grid container spacing={1} justifyContent="center">
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
                                    return <Material.Grid item>
                                        <Material.Tooltip title="Play">
                                            <Material.IconButton onClick={() => ReactInterface.run()}>
                                                <PlayArrowIcon />
                                            </Material.IconButton>
                                        </Material.Tooltip>
                                    </Material.Grid>;
                                }
                            })()
                        }
                    </Material.Grid>
                </Material.Card>
            </Material.Grid>
        </Material.Grid>
    );
}

export default IOPanel;
