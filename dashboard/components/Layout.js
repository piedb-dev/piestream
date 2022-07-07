/*
 * Copyright 2022 Singularity Data
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
import Head from 'next/head'
import Link from 'next/link';
import { useRouter } from 'next/router';

import React, { useState } from 'react';
import { styled, useTheme } from '@mui/material/styles';
import Box from '@mui/material/Box';
import Drawer from '@mui/material/Drawer';
import CssBaseline from '@mui/material/CssBaseline';
import MuiAppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import List from '@mui/material/List';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';

import MenuIcon from '@mui/icons-material/Menu';
import ChevronLeftIcon from '@mui/icons-material/ChevronLeft';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';
import DoubleArrowIcon from '@mui/icons-material/DoubleArrow';
import ViewComfyIcon from '@mui/icons-material/ViewComfy';
import InfoIcon from '@mui/icons-material/Info';

import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';

import { capitalize } from '../lib/str';

const drawerWidth = 215;

const mainPadding = 30;
const Main = styled('main', { shouldForwardProp: (prop) => prop !== 'open' })(
  ({ theme, open }) => ({
    height: "100%",
    width: "100%",
    flexGrow: 1,
    padding: `${mainPadding}px`,
    transition: theme.transitions.create('margin', {
      easing: theme.transitions.easing.sharp,
      duration: theme.transitions.duration.leavingScreen,
    }),
    marginLeft: `-${drawerWidth}px`,
    ...(open && {
      transition: theme.transitions.create('margin', {
        easing: theme.transitions.easing.easeOut,
        duration: theme.transitions.duration.enteringScreen,
      }),
      marginLeft: 0,
    }),
  }),
);

const AppBar = styled(MuiAppBar, {
  shouldForwardProp: (prop) => prop !== 'open',
})(({ theme, open }) => ({
  transition: theme.transitions.create(['margin', 'width'], {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.leavingScreen,
  }),
  ...(open && {
    width: `calc(100% - ${drawerWidth}px)`,
    marginLeft: `${drawerWidth}px`,
    transition: theme.transitions.create(['margin', 'width'], {
      easing: theme.transitions.easing.easeOut,
      duration: theme.transitions.duration.enteringScreen,
    }),
  }),
}));

const DrawerHeader = styled('div')(({ theme }) => ({
  display: 'flex',
  alignItems: 'center',
  padding: theme.spacing(0, 1),
  // necessary for content to be below app bar
  ...theme.mixins.toolbar,
  justifyContent: 'space-between',
}));

const NavBarNavigationItem = styled('div')(() => ({
  width: "100%",
  display: "flex",
  flexDirection: "row",
  alignItems: "center"
}));

const NavBarItem = ({ text, icon }) => {
  const { pathname } = useRouter();
  return (
    <Link href={`/${text}`}>
      <ListItemButton key={text} selected={pathname.slice(1) === text}>
        <NavBarNavigationItem>
          <ListItemIcon>{icon}</ListItemIcon>
          <span style={{ fontSize: "15px" }}>{capitalize(text)}</span>
        </NavBarNavigationItem>
      </ListItemButton>
    </Link>
  )
};

export default function Layout({ children }) {
  const { pathname } = useRouter();
  const theme = useTheme();
  const [open, setOpen] = useState(true);
  const handleDrawerOpen = () => {
    setOpen(true);
  };

  const handleDrawerClose = () => {
    setOpen(false);
  };

  return (
    <>
      <Head>
        <title>Dashboard | piestream</title>
        <link rel="icon" href="/singularitydata.svg" />
      </Head>
      <Box sx={{ display: 'flex', height: "100vh", width: "100vw" }}>
        <CssBaseline />
        <AppBar open={open}>
          <Toolbar>
            <IconButton
              color="inherit"
              aria-label="open drawer"
              onClick={handleDrawerOpen}
              edge="start"
              sx={{ mr: 2, ...(open && { display: 'none' }) }}
            >
              <MenuIcon />
            </IconButton>
            <div>
              {capitalize(pathname.slice(1) || ' ')}
            </div>
          </Toolbar>
        </AppBar>
        <Drawer
          sx={{
            width: drawerWidth,
            flexShrink: 0,
            '& .MuiDrawer-paper': {
              width: drawerWidth,
              boxSizing: 'border-box',
            },
          }}
          variant="persistent"
          anchor="left"
          open={open}
        >
          <DrawerHeader>

            <div style={{ display: "flex", flexDirection: "column", marginLeft: "5px" }}>
              <div style={{ display: "flex", flexDirection: "row", alignItems: "center" }}>
                <img src="/singularitydata.svg" width="20px" height="20px" />
                <span style={{ fontSize: "15px", fontWeight: "700", marginLeft: "5px" }}>piestream</span>
              </div>
              <div>
                <span style={{ fontSize: "13px" }}>Dashboard </span>
                <span style={{ fontSize: "13px" }}>v0.0.1-alpha</span>
              </div>
            </div>
            <IconButton onClick={handleDrawerClose}>
              {theme.direction === 'ltr' ? <ChevronLeftIcon /> : <ChevronRightIcon />}
            </IconButton>
          </DrawerHeader>
          <Divider />
          <List>
            <NavBarItem
              text='cluster'
              icon={<ViewComfyIcon fontSize="small" />}
            />
            <NavBarItem
              text='streaming'
              icon={<DoubleArrowIcon fontSize="small" />}
            />
          </List>
          <Divider />
          <List>
            <NavBarItem
              text='about'
              icon={<InfoIcon fontSize="small" />}
            />
          </List>
        </Drawer>
        <Main open={open}>
          <div style={{height: "68px"}}></div>
          <div style={{width: "calc(100vw - 275px)", height: "calc(100% - 68px)"}}>
            {children}
          </div>
        </Main>
      </Box>
    </>
  );
}