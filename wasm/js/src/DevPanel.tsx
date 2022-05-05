import * as React from "react";
import { useEffect, useState } from "react";
import * as Material from "@mui/material";
import { useFilePicker } from "use-file-picker";

import ReactInterface from "wasmjvm_interface/react";
import FileUploadIcon from "@mui/icons-material/FileUpload";
import SearchIcon from "@mui/icons-material/Search";

export interface DevPanelProps {
    running: boolean
}

const DevPanel: React.FC<DevPanelProps> = ({ running }) => {
    let [jarCount, setJarCount] = useState(0);

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
        <Material.Card style={{ padding: "0.5em" }}>
            <Material.Grid container spacing={1} justifyContent="center">
                {
                    (() => {
                        if (running) {
                            return <></>;
                        } else {
                            return <Material.Grid item>
                                <Material.Tooltip title="Upload Class/JAR">
                                    <Material.IconButton onClick={() => openFileSelector()}>
                                        <FileUploadIcon />
                                    </Material.IconButton>
                                </Material.Tooltip>
                            </Material.Grid>;
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
    );
}

export default DevPanel;
