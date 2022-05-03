import * as React from "react";

import { useEffect, useState } from "react";
import { useFilePicker } from "use-file-picker";
import { asyncLoad } from "./module";
import Interface from "wasmjvm_interface";

import * as Material from "@mui/material";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import FileUploadIcon from "@mui/icons-material/FileUpload";
import DownloadDoneIcon from '@mui/icons-material/DownloadDone';

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
    let [module, setModule] = useState(null);
    let [output, setOutput] = useState("");
    let [classes, setClasses] = useState<{[key: string]: IClass}>({});

    let [openFileSelector, { plainFiles }] = useFilePicker({
        accept: "*.*"
    });

    useEffect((async () => {
        if (module) {
            let main = null;

            let newClasses = await Promise.all(Object.values(plainFiles).map(async (file) => {
                let buffer = await file.arrayBuffer();
                let uint8 = new Uint8Array(buffer);

                if (file.name.endsWith(".jar")) {
                    module.load_jar(uint8);

                    return [{ path: file.name, cls: file.name, main: false }];
                } else {
                    return [];
                }
            }));

            let dict = {};
            newClasses.flat(1).forEach(entry => {
                dict[entry.cls] = entry;
            });

            if(main) {
                module.main_class(main);
                dict[main].main = true;
            }

            setClasses({...classes, ...dict})
        }
    }) as any, [plainFiles]);

    useEffect((async () => {
        let module = await asyncLoad();
        console.log(module);
        setModule(module);
        Interface.setOutput = setOutput;
    }) as any, []);

    if (module) {
        return (
            <Material.ThemeProvider theme={theme}>
                <Material.Grid container direction="column" style={{ height: "100vh" }} spacing={2}>
                    <Material.Grid item xs={2}>
                        <Material.AppBar style={{ padding: "1em" }} position="static" elevation={0}>
                            <Material.Typography>WasmJVM</Material.Typography>
                        </Material.AppBar>
                    </Material.Grid>
                    <Material.Grid container item xs={10} alignItems="flex-end" spacing={2}>
                        <Material.Grid item container direction="column" spacing={2} xs={8}>
                            <Material.Grid item xs={12}>
                                <Material.Card style={{ padding: "1em" }} square>
                                    <Material.Typography style={{ whiteSpace: "pre-line", fontFamily: "Courier New" }}>{output}</Material.Typography>
                                </Material.Card>
                            </Material.Grid>
                        </Material.Grid>
                        <Material.Grid item xs={4} container direction="column" spacing={2}>
                            <Material.Grid item container direction="column" spacing={2}>
                                <Material.Grid item>
                                    <Material.Accordion elevation={0} variant="outlined" square>
                                        <Material.AccordionSummary
                                            expandIcon={<ExpandMoreIcon />}
                                        >
                                            <Material.Typography>{`Classes - ${Object.values(classes).length}`}</Material.Typography>
                                        </Material.AccordionSummary>
                                        <Material.AccordionDetails>
                                            <Material.List>
                                                {
                                                    Object.entries(classes).map(([key, cls]) =>
                                                        <Material.ListItem key={key}>
                                                            <Material.ListItemButton onClick={() => {
                                                                let newClasses = {...classes, [key]: {...cls, main: true}};

                                                                module.main_class(cls.cls);

                                                                setClasses(newClasses);
                                                            }}>
                                                                <Material.ListItemIcon>
                                                                    {
                                                                        (() => {
                                                                            if (cls.main) {
                                                                                return (<PlayArrowIcon />);
                                                                            } else {
                                                                                return (<DownloadDoneIcon />);
                                                                            }
                                                                        })()
                                                                    }
                                                                </Material.ListItemIcon>
                                                                <Material.ListItemText primary={`${cls.cls}`} />
                                                            </Material.ListItemButton>
                                                        </Material.ListItem>
                                                    )
                                                }
                                            </Material.List>
                                            <Material.Typography></Material.Typography>
                                        </Material.AccordionDetails>
                                    </Material.Accordion>
                                </Material.Grid>
                            </Material.Grid>
                            <Material.Grid item>
                                <Material.Card style={{ padding: "0.5em" }} square>
                                    <Material.Grid container spacing={1}>
                                        <Material.Grid item>
                                            <Material.Tooltip title="Toggle Play">
                                                <Material.IconButton onClick={() => console.log(module.run())}>
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
                                    </Material.Grid>
                                </Material.Card>
                            </Material.Grid>
                        </Material.Grid>
                    </Material.Grid>
                </Material.Grid>
            </Material.ThemeProvider>
        )
    } else {
        return <></>
    }
};
