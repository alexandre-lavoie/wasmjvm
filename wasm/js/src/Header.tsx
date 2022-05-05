import * as React from "react";
import * as Material from "@mui/material";

import DevPanel from "./DevPanel";

export interface IOPanel {
    running: boolean
}

const IOPanel: React.FC<IOPanel> = ({ running }) => {
    return (
        <Material.AppBar style={{ width: "100%" }} position="static" elevation={0}>
            <Material.Grid container>
                <Material.Grid container item xs={6} alignContent="center">
                    <Material.Typography>WasmJVM</Material.Typography>
                </Material.Grid>

                <Material.Grid container item xs={6} justifyContent="flex-end">
                    <DevPanel running={running} />
                </Material.Grid>
            </Material.Grid>
        </Material.AppBar>
    );
}

export default IOPanel;
