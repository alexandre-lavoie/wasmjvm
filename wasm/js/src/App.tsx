import * as React from "react";

import { useEffect, useState } from "react";
import { useFilePicker } from "use-file-picker";
import { asyncLoad } from "./module";

import { useTheme, ThemeProvider, createTheme } from "@mui/material/styles";
import AppBar from "@mui/material/AppBar";
import IconButton from "@mui/material/IconButton";
import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import Accordion from "@mui/material/Accordion";
import AccordionSummary from "@mui/material/AccordionSummary";
import AccordionDetails from "@mui/material/AccordionDetails";
import Grid from "@mui/material/Grid";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import Tooltip from "@mui/material/Tooltip";
import FileUploadIcon from "@mui/icons-material/FileUpload";
import ClearIcon from "@mui/icons-material/Clear";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemText from "@mui/material/ListItemText";
import ListItemButton from "@mui/material/ListItemButton";

const theme = createTheme({
    palette: {
        mode: "dark"
    }
});

export default function App() {
    let [module, setModule] = useState(null);
    let [output, setOutput] = useState("");
    let [classes, setClasses] = useState([]);

    let [openFileSelector, { filesContent }] = useFilePicker({
        accept: "*.*"
    });

    useEffect(() => {
        if(module) {
            Object.values(filesContent).forEach(file => {
                setClasses([...classes, file.name]);
                module.class_load(file.content);
            });
        }
    }, [filesContent]);

    useEffect((async () => {
        let module = await asyncLoad();
        console.log(module);
        setModule(module);
    }) as any, []);

    if(module) {
        return (
            <ThemeProvider theme={theme}>
                <Grid container direction="column" style={{height: "100vh"}} spacing={2}>
                    <Grid item xs={2}>
                        <AppBar style={{padding: "1em"}} position="static" elevation={0}>
                            <Typography>WasmJVM</Typography>
                        </AppBar>
                    </Grid>
                    <Grid container item xs={10} alignItems="flex-end" spacing={2}>
                        <Grid item container direction="column" spacing={2} xs={8}>
                            <Grid item xs={12}>
                                <Card style={{padding: "1em"}} square>
                                    <Typography style={{whiteSpace: "pre-line", fontFamily: "Courier New"}}>{output}</Typography>
                                </Card>
                            </Grid>
                            <Grid item>
                                <TextField style={{width: "100%"}} />
                            </Grid>
                        </Grid>
                        <Grid item xs={4} container direction="column" spacing={2}>
                            <Grid item container direction="column" spacing={2}>
                                <Grid item>
                                    <Accordion elevation={0} variant="outlined" square>
                                        <AccordionSummary
                                            expandIcon={<ExpandMoreIcon />}
                                        >
                                            <Typography>{`Classes - ${classes.length}`}</Typography>
                                        </AccordionSummary>
                                        <AccordionDetails>
                                            <List>
                                                {
                                                    classes.map((cls, i) => 
                                                        <ListItem key={i}>
                                                            <ListItemButton>
                                                                <ListItemText primary={`${cls}`}/>
                                                            </ListItemButton>
                                                        </ListItem>
                                                    )
                                                }
                                            </List>
                                            <Typography></Typography>
                                        </AccordionDetails>
                                    </Accordion>
                                </Grid>
                                <Grid item>
                                    <Accordion elevation={0} variant="outlined" square>
                                        <AccordionSummary
                                            expandIcon={<ExpandMoreIcon />}
                                        >
                                            <Typography>Threads - 0</Typography>
                                        </AccordionSummary>
                                        <AccordionDetails>
                                            <Typography></Typography>
                                        </AccordionDetails>
                                    </Accordion>
                                </Grid>
                            </Grid>
                            <Grid item>
                                <Card style={{padding: "0.5em"}} square>
                                    <Grid container spacing={1}>
                                        <Grid item>
                                            <Tooltip title="Toggle Play">
                                                <IconButton>
                                                    <PlayArrowIcon/> 
                                                </IconButton>
                                            </Tooltip>
                                        </Grid>
                                        <Grid item>
                                            <Tooltip title="Upload Class/JAR">
                                                <IconButton onClick={() => openFileSelector()}>
                                                    <FileUploadIcon/> 
                                                </IconButton>
                                            </Tooltip>
                                        </Grid>
                                        <Grid item>
                                            <Tooltip title="Clear">
                                                <IconButton onClick={() => setOutput("")}>
                                                    <ClearIcon/> 
                                                </IconButton>
                                            </Tooltip>
                                        </Grid>
                                    </Grid>
                                </Card>
                            </Grid>
                        </Grid>
                    </Grid>
                </Grid>
            </ThemeProvider>
        )
    } else {
        return <></>
    }
};
