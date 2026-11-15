
## i3

### #79 [closed] WM bindings not work when using non English keyboard layot  (28c)
[Originally reported by DeKar] (When I change input language(english->russian) WM does not respond to hotkeys.

### #995 [closed] I3 freezes (100[pct] cpu).  (14c)
[Originally reported by layus.on@…] (I recently experienced freezes with i3. It seems to happen when I switch too fast among many workspaces. The i3 process uses 100% cpu, and needs to be killed, potentially losing unsaved work. I could not reproduce it deterministically yet.

### #926 [closed] i3 dies if there are no available outputs.  (7c)
[Originally reported by cneumann@…] (If I disable all outputs via xrandr, i3 dies with the message "No usable outputs available.". i3 should not die but wait until a now output appears.

### #451 [closed] In-place restart causes focussed window to disappear  (8c)
[Originally reported by inxsible@…] (Whenever you hit Ctrl+Shift+R, to do an in-place restart, the window under the mouse (whether focussed or not) disappears. The only way to reclaim that container is to use the WM kill (Alt+Shift+Q). Note in the screenshot, the left container had a terminal, just before I did an in-place restart. Now all you can see is the wallpaper. Focussing that empty space and then using the kill (Alt+Shift+Q) reclaims the space back.

### #718 [closed] empty container race condition  (8c)
[Originally reported by xeen] (Run this in a terminal: ``` for i in {0..1000..1}; do urxvtc -e "does_not_exist"; done ``` The faster the window closes again, the better. Therefore urxvtc outperforms urxvt. You’ll end up with a lot of empty containers that are still managed by i3 although they don’t contain any window. Restarting removes the container but has that space still being managed by i3. For completeness, I attached the irc chatlog at the bottom. The attached log is a whole i3 session, but searching for "does_not_exist" should get you started. ``` 18:41:48 <@…. 18:42:16 <@…. 18:45:12 <@… 18:47:22 <@…" 18:50:11 <@… 18:50:46 <@…. 18:53:05 <@… 18:53:32 <@… 18:53:38 <@… 18:59:25 <@…). 18:59:57 <@…. 19:01:30 <@…. 19:01:59 <@… 19:02:04 <@… 19:02:16 <@… 19:03:03 <@… 19:03:50 <@…. 19:04:48 <@… 19:05:07 <@… 19:05:55 <@…. ```

### #787 [closed] some windows not managed by i3  (13c)
[Originally reported by jhnmlkvch9] (I've been using i3 for a long time in Ubuntu 10.04 LTS. Now moved to Ubuntu 12.04 LTS and installed i3 4.1.2. I use a software that works perfectly in i3 3.d-bf1-1. However in i3 4.1.2 the help window (activated by F1 key) of this program comes up without any window decorations. It sits on top of the other tiled windows blocking them. When I click on the help window, I get the following message in the terminal: ERROR: Clicked into unknown window ?! However if I open the help window first in Gnome and then replace metacity with i3, the window works fine. Config file the default version.

### #855 [closed] System crach on automatic resizing a floating window  (8c)
[Originally reported by ishahinism@…] (I tried to make a quake like terminal with scratchpad on i3 with this commands: ``` for_window [instance="dropdown_urxvt"] move to scratchpad, border 1pixel for_window [class="URxvt"] focus; resize grow width 300px bindsym F12 [instance="dropdown_urxvt"] scratchpad show, move position 0px 0px ``` On stable version of i3 after starting the window manager, it'll crash on workspace switching. every programs are works fine but you cat switch between workspace. I found out that if I remove this line: ``` for_window [class="URxvt"] focus; resize grow width 300px ``` every thing will work fine. After that I reinstalled i3 with latest git version: 4.3-21-g718d922 (2012-10-03, branch "master"). Here the problem is a little different. After restarting the system, if I try to start i3, it'll start but all of the system will freeze. I can't even switch to …

### #891 [closed] i3-dmenu-desktop: Problems parsing .desktop files  (16c)
[Originally reported by saurabhgeek92@…] (I have a .desktop file for emacsclient. ``` [Desktop Entry] Name=Emacsclient GenericName=Text Editor Comment=Edit text MimeType=text/english;text/plain;text/x-makefile;text/x-c++hdr;text/x-c++src;text/x-chdr;text/x-csrc;text/x-java;text/x-moc;text/x-pascal;text/x-tcl;text/x-tex;application/x-shellscript;text/x-c;text/x-c++; Exec=emacsclient -c %F Icon=emacs Type=Application Terminal=false Categories=Development;TextEditor; StartupWMClass=Emacs ``` This does not open a new client but opens emacs fresh instead. Apparently the command that is run is emacs client. (with a space added), instead of emacsclient. The same .desktop file is parsed correctly by other dmenu wrappers such as dmenu-launch.

### #832 [closed] i3 doesn't respect _MOTIF_WM_HINTS  (8c)
[Originally reported by meaneye.rcf@…] (When calling gtk_window_set_decorated(*window, false), GTK+ will set _MOTIF_WM_HINTS and try to convince window manager not to draw decorations around specified window. Calling xprop on this window adds the following properties: _MOTIF_WM_HINTS(_MOTIF_WM_HINTS) = 0x2, 0x0, 0x0, 0x0, 0x0 It would be nice if i3 would just draw border around these windows. Right now i3 ignores these hints and draws both title bar and border.

### #615 [closed] pointer transition between size-mismatched outputs could be smarter  (15c)
[Originally reported by brian.grossman+i3@…] (I have two outputs, one tall on the left, one wide on the right. When I try to move my mouse from the lower part of tall one to the wide one, the mouse can't move past the edge of the tall output. It would be nice in that situation if i3 were to warp the pointer to the bottom of the wide screen. I noticed that when I try to move the mouse like that, this shows up in the log: ``` 01/12/2012 10:41:59 AM - ERROR: ERROR: No such screen ``` In contrast, when I bump up against the right side of the right-most screen no message occurs in the log, suggesting that the two edge types may already be distinct to i3.

### #780 [closed] wrong cursor placement using multiple monitors  (22c)
[Originally reported by ryzion@…] (Moving the mouse between two workspaces on separate screens can cause the cursor to be placed on the center of the first screen instead of moving to the second one. Happens regularly while moving from firefox container to any other window. Disabling focus following mouse is a workaround.

### #745 [closed] Tray Icons Disappearing on i3bar  (19c)
[Originally reported by Bruno BRAGA <bruno.braga@…>] (I am experiencing this annoying behaviour for some icons in the tray bar. They sometimes disappear out of nowhere, sometimes after coming back from suspend, or when I reload the i3 config (by hitting Mod+Shift+R). It always happens with checkgmail/xchat icons, but intermittently with Skype as well. I tried to check the logs, but there was nothing there. Even though the icons are gone, the applications are still running. More details of this are discussed in the mailing list: http://infra.in.zekjur.net/archives/i3-discuss/2012-June/000745.html From the logs, it looks like the problem from the mailing list: ``` [src/xcb.c:445] ERROR: Window 022002d7 violates the XEMBED protocol, _XEMBED_INFO not set ``` (complete log is attached).

### #709 [closed] Java application gets frequently blank windows  (17c)
[Originally reported by lemaire.adrien@…] (Problem with a java application. Will fail rendering after some time, while running with chromium and urxvt+tmux

### #934 [closed] JOptionPane.showMessageDialog shows empty Windows  (14c)
[Originally reported by i3-new-ticket] (Some Java Applications randomly show empty Message Dialogs. Setting wmname to LG3D does not help. In my Application i just call JOptionPane.showMessageDialog I use: Oracle java 1.7.0_11 OpenJDK 1.7.0_03

### #594 [closed] mplayer does not display ass subtitles when fullscreen and -vo vdpau  (9c)
[Originally reported by alexander+i3@…](When I play a video file with embedded ass subtitles %28and mplayer enabled to use them with) and my video-out set to vdpau, then the subtitles disappear when switching mplayer into fullscreen through mplayer's internal fullscreen mode. With non-ass subtitles, video-out not set to vdpau, other window managers (I tried openbox) or fullscreen through i3's fullscreen setting the subtitles do not disappear. So the steps to reproduce are: -Use an nVidia card with support for vdpau, have drivers, vdpau libs, etc installed -Have an video file with embedded ass subtitles -Start mplayer under i3 with the flags "-ass" and "-vo vdpau", enable subtitles (pressing [j]) -Switch mplayer into fullscreen using its internal mode (pressing [f]) => subtitles disappear -Switch mplayer into fullscreen using i3's mode (pressing [alt]/[win] + [f]) => subtitles do not …

### #1019 [closed] Sometimes can't move focus between windows when mouse is on their border  (7c)
[Originally reported by jebaum@…] (When the mouse is on the border of two windows, sometimse focus gets 'stuck' on one of them. In the video I made, at :15 and again at :37, each time focus flashes back and forth I am pressing my focus left key exactly one time. When I press it, focus very briefly flashes to the left window before returning to the right. Once i3 is stuck in this behavior, I can only get it to stop by moving the mouse pointer. I can fairly reliably, but not 100% reliably, reproduce this behavior using the steps in the video, provided the mouse is in the center of the screen. I get the mouse to the center of the screen by pressing my focus right key on a multimonitor setup with an empty workspace, and then focus left. This places the mouse in the center of the screen automatically. This behavior can occur with any arbitrary number of windows if the mouse is on the border …

### #240 [closed] Three Column Resize Results in Lost Width  (6c)
[Originally reported by swdunlop@…] (When resizing three windows in a three column layout, the width of the workspace may differ unexpectedly from the width of the display. This difference will persist until i3 is restarted. '''i3 Version:''' i3 version f70530d (2010-07-03) © 2009-2010 Michael Stapelberg and contributors '''OS Version:''' Ubuntu Lucid Lynx (amd64) '''Reproduction:''' 1) Create three windows; for my testing, I used xterms, but this has been observed with Gtk windows as well. 2) Divide the three windows into three columns, one window per column. 3) Observe that the three use roughly the same space, and consume the full width of the display. 4) Resize columns by moving the resize sash between the second and the third columns. 5) Observe that the second and third columns have resized and continue to consume the full width of the display. 6) Resize columns by moving the …


## awesome

### #22 [closed] Regression: dialogs are not handled correctly anymore  (5c)
https://github.com/awesomeWM/awesome/commit/b4c83d0e702168c5bbcf3becaad89897c3f06d70 breaks dialogs. They now appear to get handled as tiled first, before being set to floating state (not sure about that latter, since they not necessarily are displayed in the taskbar). TEST CASE: 1. in GVim (in a tiled layout) 2. `:call inputdialog("foo")` The dialog box will appear tiled (oversized) as master. It is probably caused by this change: https://github.com/awesomeWM/awesome/commit/b4c83d0e702168c5bbcf3becaad89897c3f06d70#diff-7de93d6ae06f822664ee72cf4127a1e8L616

### #63 [closed] AltGr shortcut combinations not work  (16c)
i use dvorak layout(but it is not important for this question) and on my 3 level of layout i have useful symbols like digits and arrows so AltGr + a = 4 AltGr + o = 5 AltGr + e = 6 AltGr + h = left AltGr + t = Down AltGr + n = UP AltGr + s = Right But, in awesome wm 3 level doesn't work, and i don't understand why. for example awful.key({ modkey }, "Left", awful.tag.viewprev ), awful.key({ modkey }, "Right", awful.tag.viewnext ), it's works with simple arrows on keyboard, but not works with level3 arrows.

### #130 [closed] Tag is not losing "urgent" state  (35c)
After one of the patches merged yesterday tags are not losing "urgent" state after receiving it.

### #135 [closed] Closing programs in the systray makes keyboard unresponsive  (12c)
Since awesome 3.5.6, closing a program from the systray (right click->close) makes the keyboard totally unresponsive. No keys work, neither the default ones nor my own keybinds. Tested with Dropbox and Google-MusicManager (I have no other programs that make use of the systray). To restore keyboard's functionality it's necessary to change tag by clicking with the mouse on the statusbar. No other action is sufficient to make the keyboard responsive again. ``` awesome v3.5.6 (For Those About To Rock) • Build: Jan 10 2015 23:18:34 for x86_64 by gcc version 4.9.2 (builduser@) • Compiled against Lua 5.2.3 (running with Lua 5.2) • D-Bus support: ✔ ```

### #169 [open] Key release signals not emitted for modifier keys  (28c)
Signals for the modifier keys (Alt_L, Ctrl_L, Super_L, etc.) are emitted only on key press events, not on key release events. I added ``` awful.key({ modkey }, "Alt_R", nil) ``` to `globalkeys`, then added the lines ``` for i,v in pairs(globalkeys) do if v.key == "Alt_R" then v:connect_signal("press", function(...) print("press signal received") end) v:connect_signal("release", function(...) print("release signal received") end) end end ``` Running awesome inside Xephyr, I see the press signals but not the releases. Other keys such as alphanumerics work as expected.

### #170 [closed] Focus problems in multi monitor setup with Google Chrome / Chromium  (24c)
**Problem:** I use awesomewm v3.5.6 and have the following problem in a multi-monitor setup: When I use chromium browser I cannot rearrange tabs using the mouse. When I try to drag a tab I instantly get a new window with that tab. After that it is not possible to move this tab back into the main window. Another issue: in the bookmark manager (of chromium) it is not possible to reagange the bookmarks using drak and drop either. **Why I think this is an awesomewm issue:** Today I found, that these issues only occur on screen two and three. On screen one everything works fine. More precisely: it only works on the primary monitor (see below) **What I did to setup monitors:** `xrandr --output eDP1 --auto --primary --output DP4 --auto --right-of eDP1 --output HDMI1 --auto --right-of DP4` the screen where I can move tabs moves with the `--primary` flag

### #245 [closed] Maximized Firefox windows not restored properly after fullscreen (F11)  (6c)
TEST CASE: 1. Tiled layout, with a maximized Firefox window (Mod4-m). 2. Press F11 to switch Firefox into fullscreen mode. 3. Press F11 again. There will be a gap (~35px) between awesome's taskbar and the Firefox window. The Firefox window will not be recognized as "maximized" anymore, but only "floating". The bottom of the Firefox window is aligned to the bottom (and not overlapping), reducing the window's height effectively (by ~35px) after leaving fullscreen mode. The same behavior occurs when using awesome's fullscreen shortcut (`mod4+f`).

### #203 [closed] In dual monitor configuration, maximization of a window in screen2 moves window to screen1   (5c)
Hi, I have a dual monitor configuration (laptop LVDS1 in 1366x768 + external VGA1 in 1600x900). In my .config/awesome/rc.lua , I've added two entries: { rule = { class = "Firefox" }, properties = { maximized_horizontal = true, maximized_vertical = true } }, { rule = { class = "Firefox" }, properties = { tag = tags[screen.count()][6] } }, So Firefox is supposed to run maximized on screen 2 (VGA1). It opens on screen2, but it is not maximized, so I try to maximize its window using my combo "Ctl-Alt-a", which is defined as: awful.key({ modkey, "Control" }, "a", function (c) c.maximized_horizontal = not c.maximized_horizontal c.maximized_vertical = not c.maximized_vertical end) But this moves the Firefox window to the other screen (laptop LVDS1). I'm wrong (*) or is it actually a bug? PS: awesome v3.4.15 (Never Gonna Give You Up) • Build: Jan 23 2014 14:28:37 for x86_64 by gcc version …

### #196 [closed] Panel not responding mouse when I switching to russian keyboard  (29c)
Good morning! I found bug: when I'm switching to Russian keyboard, top panel not responding.

### #219 [closed] awful.widget.prompt and menubar do not support russian (cyrillic) input  (5c)
When I trigger these widgets (Meta+R, Meta+P by default) and try to type text, input is always in English, even if I have another keyboard layout chosen (for example, if I use russian keyboard layout). Thus I cannot access russian-named files using awful.widget.prompt/menubar, for example.

### #230 [closed] Awesome uses high CPU when key held down  (4c)
I'm using the ['standard'](http://awesome.naquadah.org/wiki/Conky_HUD) way of getting Conky to work with awesome. Thing that's weird is that when I hold down the key to activate it, Conky zooms up in its CPU usage. That makes sense since a function is being called every time they key repeats, BUT, this didn't happen on 3.4.15. Just showed up when I switched to 3.5.6. Problem? Screws up my 'top' display in Conky if nothing else. ![screenshot from 2015-05-16 07 49 43](https://cloud.githubusercontent.com/assets/3937658/7662556/2e1ce47a-fba1-11e4-9e7f-549184b42a7d.png)

### #253 [closed] Mouse gets crazy over root window and jumps around randomly  (3c)
I am using Awesome with Arch Linux on a new machine. Things appear to work fine on Gnome but with Awesome I have this weird problem that my mouse behaves normally over application windows, but if I leave it over the root window it starts moving around randomly, jumping here and there a few times every second. The fact that happens only over the root window makes me think the window manager might have something to do with it. Any ideas? The mouse is actually not completely normal otherwise. Sometimes the movement gets a chopped, like motion events are being dropper, perhaps.

### #241 [closed] rules doesn't honor "custom" WM_CLASS  (2c)
MPV have a feature where you can tell it which WM_CLASS it should expose if I run `mpv` with `--x11-name` argument and provide some kind of custom class name to - awesome doesn't honor this name but only the original, though xprop shows me 2 class name, the custom and the original. ``` alex in ~ λ mpv --x11-name Downloads/KAAR.mp4 Playing: Downloads/KAAR.mp4 (+) Video --vid=1 (vp9) VO: [opengl] 852x480 yuv420p alex in ~ λ alex in ~ λ xprop | grep WM_CLASS WM_CLASS(STRING) = "mpvldb", "mpv" ```

### #236 [closed] claws mail doesn't obey tag assignment from awful.rules  (19c)
I have awesome configured so that claws-mail always starts on tag 3 on my right screen. When I upgraded to Awesome 3.5.6, I noticed that claws mail no longer obeyed my configuration, instead going to tag 1 on my left screen. Downgrading to 3.5.5 fixes the problem, and this behavior is present in master (bafa12c as of this writing) I did some digging into the codebase, and it seems the responsible commit is f4a7b2d. I don't know if claws-mail is doing something silly, but I thought I would bring this to team awesome's attention. Is there anything I can try to help provide more information? I am also happy to help make the code changes necessary to fix this issue. My configuration is here, by the way: https://github.com/hoelzro/awesome-config

### #111 [closed] "less" ("<") in globalkeys mapping breaks "comma" (",") in clientkeys  (3c)
When using `<` ("less") in a global keybinding, `,` (comma) in a client keybinding does not work. ``` globalkeys = awful.util.table.join( .... awful.key({ modkey }, "less", function () naughty.notify({text="less"}) end), clientkeys = awful.util.table.join( ... awful.key({ modkey }, ",", function(c) naughty.notify({text="comma"}) end), ``` I've tried using `<` and `comma` instead. I am using a German keyboard layout, and `<` is next to the left shift and comma right next to `m`. Output from `xev`: For `<`: ``` KeyRelease event, serial 34, synthetic NO, window 0x3a00001, root 0x94, subw 0x0, time 393382277, (438,630), root:(439,631), state 0x10, keycode 94 (keysym 0x3c, less), same_screen YES, XLookupString gives 1 bytes: (3c) "<" XFilterEvent returns: False ``` For `,`: ``` KeyRelease event, serial 34, synthetic NO, window 0x3a00001, root 0x94, subw 0x0, time 393403254, (103,14), …

### #22 [closed] Regression: dialogs are not handled correctly anymore  (5c)
https://github.com/awesomeWM/awesome/commit/b4c83d0e702168c5bbcf3becaad89897c3f06d70 breaks dialogs. They now appear to get handled as tiled first, before being set to floating state (not sure about that latter, since they not necessarily are displayed in the taskbar). TEST CASE: 1. in GVim (in a tiled layout) 2. `:call inputdialog("foo")` The dialog box will appear tiled (oversized) as master. It is probably caused by this change: https://github.com/awesomeWM/awesome/commit/b4c83d0e702168c5bbcf3becaad89897c3f06d70#diff-7de93d6ae06f822664ee72cf4127a1e8L616

### #127 [closed] Crash due to lgi bug  (1c)
Hi, I'll let a wall of text speak for itself: ``` E: awesome: signal_fatal:254: signal 11, dumping backtrace ./awesome(backtrace_get+0x41) [0x423f12] ./awesome() [0x40ccea] /lib/x86_64-linux-gnu/libc.so.6(+0x35180) [0x7fa5bec4d180] /lib/x86_64-linux-gnu/libc.so.6(cfree+0x14) [0x7fa5bec944a4] /lib/x86_64-linux-gnu/libglib-2.0.so.0(g_string_free+0x3b) [0x7fa5c146a83b] /usr/lib/x86_64-linux-gnu/libgobject-2.0.so.0(g_boxed_free+0x13b) [0x7fa5c171d70b] /usr/lib/x86_64-linux-gnu/lua/5.1/lgi/corelgilua51.so(+0xed5e) [0x7fa5baf8cd5e] /usr/lib/x86_64-linux-gnu/lua/5.1/lgi/corelgilua51.so(lgi_marshal_2lua+0x3cc) [0x7fa5baf9020c] /usr/lib/x86_64-linux-gnu/lua/5.1/lgi/corelgilua51.so(+0x82ca) [0x7fa5baf862ca] /usr/lib/x86_64-linux-gnu/lua/5.1/lgi/corelgilua51.so(+0xa1c6) [0x7fa5baf881c6] /usr/lib/x86_64-linux-gnu/liblua5.1.so.0(+0xc1f0) [0x7fa5bf5161f0] /usr/lib/x86_64-linux- …

### #134 [closed] An alleged race condition on client focus  (7c)
Hi, On current master there seems to be a race condition, that sometimes makes focus go wrong on new clients. This was made manifest after #118, but I guess it was there hidden before that. I have configured awesome to set focus on new clients via a rule `focus = true`. For me, it doesn't work for the first client in a tag, but does work when the tag already has a client. After introducing some printf's in the right places, I observed this sequence of calls after creating a new client: **EDIT**: It turns out the client always gets focus! What happens in case A below is that the focus signal fails to set the border color for the client. I was just looking at that to tell when a client has focus or not... sorry! A. Case in which focus does not work (no clients in a tag) ``` 1. client_focus 2. client_focus_update (focused_new = 1) 3. client_focus 4. client_focus_update (focused_new = 0) …

### #162 [closed] Alpha channel ignored on borders  (11c)
When specifying border colors in an awesome theme, the alpha channel is ignored. Far as I can tell, only the border colors are affected, but I have only played around with the values supplied in the default theme. #### Current behavior: Consider this screenshot: https://paste.xinu.at/ckn2U/ For the titlebar, the alpha channel works, with the color being "#7fff00b4". The border, being assigned a color of "#7fff0000", should be invisible, but isn't transparent at all. #### Expected behavior: For the borders to honor the alpha channel, which for the supplied example values would mean the border would be fully transparent. PS: Someone in the IRC channel implied that the alpha channel is actively stripped/otherwisely ignored in the C part of awesome; And while the lua part of awesome does have a function to strip the alpha channel from a color, I haven't seen it applied to any border …

### #171 [closed] RFC: Set c.border_width to 0 when maximized  (22c)
Someone asked for how to do this on IRC. I think it is a good idea, should we do this by default? Addition to `rc.lua`: ``` lua client.connect_signal("property::maximized", function(c) c.border_width = c.maximized and 0 or beautiful.border_width end) ```

### #231 [open] float/max/full/minimize should be set on a per-tag basis  (4c)
I'm creating an issue for this feature request / bug / design reconsideration. In summary, it appears that window status characteristics such as floating, maximized, and fullscreen are not stored on a per-tag specific basis. as I've described below, tags appear to be by design representations of collections of clients; however, it clients share a status among tags, you cannot represent them differently. one thing that came up in discussion is what the default value of the client should be. I proposed that when you add a tag to a client, the rules for float/max/full from awful.rules should be reapplied or all except the tag property (for obvious reasons). as a side note: I would also argue that clients should also have the capability to be tagged on multiple screens (with obvious restrictions that no window can be "unminimized" on more than one screen) but this is low priority. I've …


## bspwm

### #16 [closed] Mouse bindings doesnt work with numlock on  (14c)
1. Turn numlock on 2. Use mouse bindings

### #13 [closed] makefile: build binaries when header files change  (2c)

### #48 [closed] gtk3 floating menu issue  (17c)
Steps to reproduce: -> open a GTK3 application. -> set it to floating. -> click on a menu item. -> menu initially appears behind application. Tested with Evince, Transmission, and a few other GTK3 applications.

### #62 [closed] startx results in a segfault   (11c)
bspwm stopped working on my workstation as of 2c363903a9. <code>startx</code> fails with the following in <code>/var/log/syslog</code>: <pre> Sep 23 20:28:54 workstation acpid: client connected from 4092[0:0] Sep 23 20:28:54 workstation acpid: 1 client rule loaded Sep 23 20:28:55 workstation kernel: [ 107.050820] NVRM: GPU at 0000:01:00: GPU-3736b905-88ce-b01a-e7f9-3e55a1fffb4c Sep 23 20:28:55 workstation acpid: client connected from 4092[0:0] Sep 23 20:28:55 workstation acpid: 1 client rule loaded Sep 23 20:28:55 workstation kernel: [ 107.387652] bspwm[4095]: segfault at 1f2000001f1 ip 000000000040d62a sp 00007fff4ab3dc40 error 4 in bspwm[400000+11000] </pre> This is a dual-head workstation. I tested with an empty bspwmrc and a reduced .xinitrc: <pre> [ -e "$PANEL_FIFO" ] && rm "$PANEL_FIFO" mkfifo "$PANEL_FIFO" exec bspwm -s "$PANEL_FIFO" -p W </pre> The same version of bspwm works …

### #67 [closed] Full screen games create new, unwanted Desktops, and re-order exisiting Desktops  (9c)
After launching a game in full-screen mode, a new desktop is created ("Desktop3") and the order of existing desktops is changed. Ideally, the desktop configuration would remain unchanged. Steps to reproduce: 1) Start bspwm with 4 desktops named: one, two, three four. 2) Launch game in full-screen mode via steam. 3) Exit game Expected: The number, names and order of desktops remains the same as in (1). Actual: Desktops change to: "two, three, four, one, Desktop03" (or similar) Environment: Ubuntu Dual monitors (left: portrait, right: landscape)

### #88 [closed] Possible rules deadlock  (8c)
Calling bspc from the rules script leads to a deadlock.

### #91 [closed] Moved desktops pushed to back of desktop list  (7c)
I'm used to moving desktops between monitors from xmonad (and more recently i3), and while this is possible in bspwm it quickly shuffles the desktops around in a way that's a bit unintuitive in my opinion. Say I have two monitors, M1 and M2, and desktops 1 2 3 4 5. M1 has desktops 1 2 3 4, M2 has desktop 5. Say desktop 1 is focused, and I want to move it to M2. After doing: ``` bspc desktop focused -m M2 ``` the desktop list now looks like: 2 3 4 5 1, and I now get to desktop 1 by focusing the ^5:th desktop, which doesn't make that much sense if I've bound keys 1-5 to jump to desktops ^{1-5}. Also the desktops will appear in this weird order in the panel. Granted to fix the keybind problem I could just focus the desktop named "3" when hotkey 3 is pressed, and could perhaps script up something that sorts the desktops in the panel, but would it not be easier to always keep the order of …

### #101 [closed] Floating windows go to position (0, 0), even when that location isn't visible on any of my displays  (3c)
My leftmost monitor has a vertical offset, but floating windows go to (0, 0), and as a result, I can't see them. Screenshot: http://i.imgur.com/6JCSAT8.png (calculator is floating, bar + terminals show where my displays really are.)

### #103 [closed] Internal State not being written to PANEL_FIFO  (3c)
I'm using example panel scripts, bspwm is getting started with a PANEL_FIFO, etc.: bspwm -s /home/jonlorusso/.cache/bspwm-session.Aucgup/panel-fifo -p W Any ideas?

### #108 [closed] Issue replaying captured events (discovered using feh)  (2c)
Steps to reproduce: In shxkdrc, use sxhkd mouse pointer focus binding from man page example: ~button1 bspc pointer -g focus - launch feh - right click to bring up menu - click exit It should take a few clicks to get the app to exit (may take a few tries). (tested in virtualbox with a minimal arch install running just sxhkd and bspwm from xinitrc with default configurations)

### #111 [closed] Obey floating window size hints.  (6c)
While this doesn't really make sense for tiled windows, floating windows should really obey minimum/maximum size hints. ``` $ xprop ... WM_NORMAL_HINTS(WM_SIZE_HINTS): user specified location: 363, 144 user specified size: 222 by 172 program specified minimum size: 222 by 172 program specified maximum size: 222 by 172 window gravity: NorthWest ```

### #112 [closed] Windows being rendered in multiple workspaces  (17c)
After a recent update, I've started having issues with some programs' windows being rendered in other workspaces than the one they're currently open in. Only happens with OpenGL viewports, I think (happens with games, and websites in chromium. Not sure what it renders the websites with, but the chromium UI is not rendered to other workspaces.) It only happens if I switch from a workspace with a OpenGL viewport (or whatever) into another workspace with open windows. If I go directly to a workspace without windows, it stops drawing the OpenGL viewport, then I am free to go to my desired workspace. If you need more information, or want me to try to pin-point the version causing the issue, I'll be happy to help. Edit: Yeah, I just noticed I was running a version 0.8.6, so I updated it. Still got the same issue though, but obviously it wasn't an update of bspwm that broke it. I don't know …

### #129 [closed] Class/Instance names truncated to 32 characters.  (8c)
Chromium extensions have a longer identifier. I can workaround by truncating my rules but I feel that extending it a bit may be the better option long term. Example for GMail extension: crx_pjkljhegncpnkpknbcohdijeoej crx_pjkljhegncpnkpknbcohdijeoejaedia

### #139 [open] Floating window stuck beneath tiling windows  (16c)
To recreate the problem: Have 2 monitors, Have one window tiled and one floating. Startup a youtube link in fullscreen and move your cursor to the second monitor and click around there (you might need a tiled window there as well) Go back to the second monitor and remove the fullscreen. Now all your floating windows will be under the tiling windows :/

### #145 [closed] Chromium new popups and windows open in original workspace  (12c)
I'm not sure if this is a bug with BSPWM or with Chromium, but it's quite unique. To reproduce: 1. Open Chromium on workspace 2 (without any other Chromiums open, preferrably right after boot, so it's the first Chromium). 2. Once it's open move it to workspace 1. 3. Now change focus to workspace 1. 4. Now open a new Chromium window. At this point for me this opens a new Chromium window on workspace 2. Not sure why. I'm thinking this is BSPWM keeping track of where to open a new windows by a certain window based on it's original workspace. Just an idea though. :)

### #148 [closed] remove_unplugged_monitors is not working  (6c)
I have a laptop that is sometimes connected to an external monitor. I've configured bspwm to remove unplugged and disabled monitors, but bspwm does not remove the external monitor when it is disconnected, Excerpt from bspwmrc: ``` bspc config remove_disabled_monitors true bspc config remove_unplugged_monitors true ``` 2 screens: built-in laptop screen + external monitor: ``` $ bspc query -T && xrandr a 1920x1080+2560+0 28,0,0,0 * one 6 12 0,0,0,0 T - * m URxvt urxvt 0x2C00009 6 724x412+3152+328 R -------- * two 6 12 0,0,0,0 T - m URxvt urxvt 0x600009 6 724x412+3184+184 R -------- * three 6 12 0,0,0,0 T - m URxvt urxvt 0x2600009 6 724x412+591+327 L -------- * four 6 12 0,0,0,0 T - V a 0.520000 H m 0.480000 a Google-chrome google-chrome 0x34000B4 6 1200x725-1855+342 R -------- * c URxvt urxvt 0x800009 6 724x412-1969+326 L -------- H c 0.520000 m URxvt urxvt 0x2800009 6 724x412-1966+329 R …

### #60 [closed] Moving the pointer to a different monitor does not move monitor focus.  (7c)
When moving the pointer to an unfocused monitor, the monitor does not become focused until a window in said monitor is focused. An inverse of `pointer_follows_monitor` (`monitor_focus_follow_pointer`) would be nice. An alternative solution is to make `focus_follows_pointer` apply to monitor's (or their root windows) as well. It should also be noted that clicking on the root window will focus that monitor.

### #71 [closed] bspwm desktop -f fails when there is only 1 desktop on a monitor, in a multi-monitor configuration  (6c)
WIth two montors, and 1 desktop on the left monitor and 3 desktops on the right monitor, bspc cannot change focus to the desktop on the first monitor via the `bspc desktop` command, but it can change focus to the other desktops on the second monitor. Does not work: bspc desktop ^1 -f Works: bspc desktop ^[2-4] -f

### #56 [closed] xkill does not work  (6c)
Attempting to kill applications using <code>xkill</code> does not seem to have any effect. Output is as follows: <pre>$ xkill Select the window whose client you wish to kill with button 1.... $ </pre>

### #113 [closed] Steam bleeding trough desktops  (2c)
When i start up steam and then switch top another desktop. Steam's image bleeds through. The other programs only shows up if i move the mouse over. But the steam image keeps updating where the mouse aint.

### #115 [closed] Compton Opacity Problem  (2c)
Hello, I have a problem with bspwm and compton. I have set compton to always have opacity on all windows but when i change desktop the opacity is only present on the last focused window as i switch desktops back and forth.

### #137 [closed] Strange behaviour on different external monitor swap  (11c)
I am not sure if this is problem with bspwm or fbpanel, but I decided to submit it here for now. My bspwm monitor/desktop configuration is as following: ``` bspc desktop ^1 -n 1 bspc monitor LVDS1 -d 1 2 3 4 bspc monitor VGA1 -d 5 6 7 8 bspc config top_padding 14 ``` I am on a laptop and I have 2 different display monitors which I plugin in and out on same port. When I start with monitor A plugged in, my configuration is fine. Now I unplug Monitor A and plug in Monitor B. Bspwm creates desktop 9 (named `Desktop4` for Monitor B). Now if I unplug Monitor B and plug in Monitor A back, it creates another new desktop for Monitor A while moving all the previously created desktops to LVDS1 monitor. Is there any way to avoid this? I would like to achieve one of the two possible behavious: 1. Attach desktops to Port, so no matter what monitor I plugin to VGA1 port, the desktops will belong to …

### #140 [closed] Setting "persistent_monitors" doesn't work for AV receiver  (5c)
I just updated bspwm to the most recent git version as of today, with the new "persistent_monitors" option. When setting this option, I can now disconnect and reconnect a monitor and maintain my normal desktop layout, but turning off a connected AV receiver will still produce the previous behavior of removed monitors. My setup is three monitors in my office, and a receiver and TV in the living room, connected with a long HDMI cable. I mirror the receiver/TV to the middle monitor, to use XBMC, etc., on the bigger screen. The current behavior is rather frustrating. If I have started bspwm without the TV output activated, and then use xrandr to turn it on, I lose all but one of the desktops on the middle monitor, and the monitors are rearranged to be out of order. If I start bspwm with the TV/receiver already turned on, then turning them off will produce the same undesirable effects. I …

### #90 [closed] bspwm does not notice changed monitor setup  (5c)
I sometimes switch from my 1080p multi monitor setup (one is laptop monitor, LVDS-0 in xrandr, other is connected via HDMI-0) to my 720p projector (connected via VGA-0). I toggle between these setups by using xrandr. However bspwm seems to be stuck with the old monitor configuration, monitor names/resolutions still are the old ones after switching to the 720p projector, and desktops still belong to the old monitor names. This results in all windows being too big, and the desktops that were on the laptop screen can't be seen at all on the projector until I move the desktop manually. (scripting this is not an option when the last desktop can't be moved) Here's the output of bspc query -T before disabling both monitors: [rasse@lappy:~]% bspc query -T LVDS-0 1920x1080+1920+0 18,0,0,0 I 1 30 T - V 1 30 T - * m Steam 0x2C00010 1 470x352+2645+364 R --------- * HDMI-0 1920x1080+0+0 18,0,0,0 * …

### #72 [closed] Inconsistent floating behavior  (2c)
There are two different floating behaviors: action: toggle float on an existing tiled window result: floating windows float above the tree of other tiled windows. ``` +-------------------------+ +-------------------------+ |1 |2 | |1 | | | | Toggle | +------+ | | | | Float | |2 | | | | | ------> | | | | | | | | | | | | | | | +------+ | | | | | | +-------------------------+ +-------------------------+ ``` action: launch new application with `--floating` rule result: floating windows take real space in the window layout as if it were tiled action: alt+mouse1 drag floating window between monitors result: floating windows take real space in the window layout as if it were tiled ``` +-------------------------+ +-------------------------+ |1 | |EMPTY |1 | | | Launch |SPACE +------+ | | | Floating | |2 | | | | App | | | | | | ------> | | | | | | | +------+ | | | | | | …

### #92 [closed] Pipes not being closed by handle_rules  (2c)
Some users reported bspwm spawning a number of shells without closing them but a simple `pclose(results);` in handle_rules should fix it... Attached is an image of what I'm talking about. ![shell](https://f.cloud.github.com/assets/1404195/1487565/4a4e3b74-4736-11e3-8cf1-8aece34945ae.png) After adding in the code from my earlier pull request (https://github.com/windelicato/bspwm/compare/88-fix-deadlock#diff-d01b5bb7484279737a51f61e61c90647R119) the users reported this issue fixed. I know you're reworking a lot of this code, so just something to keep in mind as you use forward :smile:

### #119 [closed] proposal to change `bspc desktop --to-monitor` behaviour  (2c)
`bspc desktop --to-monitor` command should force focus the moved desktop. currently it does the move operation on the background. suppose you have 3 monitors and you want to move a desktop from the leftmost monitor to the rightmost monitor via `bspc desktop --to-monitor DIRECTION`. currently this operation is too cumbersome because of the 'move, and make it disappear' behaviour. like this: 1. press shortcut. desktop is moved to middle monitor (oops it's gone). 2. go to middle monitor. 3. find and switch to that desktop. 4. press shortcut. desktop is moved to the desired monitor (gone again). 5. go to desired monitor. 6. find and switch to that desktop. were it to focus the moved desktop immediately you could have done the above operation in just 2 shortcuts.

### #123 [closed] Error reporting  (2c)
Invalid commands should report an error on STDERR (not failed commands, just one's that don't parse). They should probably also return a different error code (2?). This would help with prevent problems like [this](https://bbs.archlinux.org/viewtopic.php?pid=1382287#p1382287).


## xmonad

### #1 [closed] Stable numlock handling.  (3c)
Originally reported on Google Code with ID 1 ``` Handling of input in the presence of numlock is fragile. Come up with a proper solutoin to this for 0.2 ``` Reported by `dons00` on 2007-04-29 05:11:14

### #3 [closed] xmonad steals input if Config.hs refers to unbound keys  (2c)
Originally reported on Google Code with ID 3 ``` (As discussed briefly on #xmonad on 2007-04-28...) If Config.hs contains the following line: , ((none, xK_F20 ), spawn "urxvt") and F20 is unmapped (ie no key is bound to it, no xmodmap entry, etc.) then xmonad seems to steal all input. That is, all other xmonad bindings seem to work fine (in particular, eg, mod-p for dmenu and mod-shift-return to launch a terminal) but no launched applications receive keypresses. If you launch urxvt (eg via dmenu), then on every keypress that window flickers as if losing focus momentarily (xterm doesn't seem to do this). I have observed similar behaviour in dwm and wmii, so perhaps the problem is mine rather than xmonad's. Furthermore, jcreigh was apparently unable to reproduce this behaviour. In dwm a config.h line as follows causes the same behaviour: { None, XK_F20, spawn, "exec xterm" }, \ (In …

### #4 [closed] Separate focus for tiled and floating layers  (26c)
Originally reported on Google Code with ID 4 ``` Add support for floating windows. Stages: - modify StackSet to store floating status - store preferred window position in XState - update layout algorithm - add keybindings to float/unfloat windows ``` Reported by `SpencerJanssen` on 2007-05-01 21:06:45

### #5 [closed] Floating window resize issue  (18c)
So making a summary for this isn't easy. Here's the setup: - two monitors - different resolutions (important!) - keyboard focus on A - cursor on B When using a manage hook to float dialogs (`isDialog --> doFloat`), it is stored in the layout according to the cursor screen resolution (B). When the window actually appears, it shows up next to the currently focused window (on A). However, because the resolutions are different, the ratio stored in XMonad for its location is a different size. When the window appears, XMonad resizes it to be the same _ratio_, but that means the _resolution_ is different. Dialogs (possibly made worse due to improper size hints) then either get inflated with empty space or squished together with overlapping widgets. The fix should be to determine the ratio of new windows based on where they will appear rather than the cursor-current screen resolution. See …

### #12 [closed] Restart fails with large serialized state .  (3c)
I'm using lots of layout modifiers.. the serialized state for a running sessions is usually about 150-200kb after some hours.. I haven't really programmed any Haskell in recent years so I just bashed this out as quickly as possible inside my own config. ``` hs -- WIP restart state via file modification -- TODO: handle file system errors properly -- TODO: convert into xmonad patch -- TODO: add --resumefile argument into xmonad and tempfile as state transfer holder(?) stateFile = "xmonadargs.txt" restartFile :: String -> Bool -> X () restartFile prog resume = do broadcastMessage ReleaseResources io . flush =<< asks display let wsData = show . W.mapLayout show . windowset maybeShow (t, Right (PersistentExtension ext)) = Just (t, show ext) maybeShow (t, Left str) = Just (t, str) maybeShow _ = Nothing extState = return . show . mapMaybe maybeShow . M.toList . extensibleState args <- if …

### #15 [closed] avoidStruts isn't working on workspace 1  (13c)
I'm using xmonad 0.12, xmonad-contrib 0.12 and xmobar on archlinux. After the upgrade from 0.11 to 0.12 avoidStruts isn't working on workspace 1. On all other workspaces my config works like expected. my xmonad config ``` import XMonad import System.Exit import qualified XMonad.StackSet as W import qualified Data.Map as M import qualified XMonad.Actions.FlexibleResize as Flex import XMonad.Util.Run import XMonad.Actions.CycleWS import Graphics.X11.ExtraTypes.XF86 import XMonad.Hooks.DynamicLog import XMonad.Hooks.ManageDocks import XMonad.Hooks.ManageHelpers import XMonad.Hooks.EwmhDesktops import XMonad.Hooks.SetWMName import XMonad.Layout.Named import XMonad.Layout.TrackFloating import XMonad.Layout.ResizableTile flexoKeyBindings conf@(XConfig {XMonad.modMask = modm}) = M.fromList $ [ ((modm, xK_Return), spawn "termite"), ((modm, xK_r), spawn "rofi -terminal termite -modi run -show …

### #21 [closed] Regression: dzen2 now gets obscured by windows  (24c)
Since upgrading xmonad from 0.11.1 to 0.12 and xmonad-contrib from 0.11.4 to 0.12, I noticed that on the first workspace dzen2-0.9.5 would always overlap with the windows, causing it to become obscured whenever a tiled window is present. But this problem does not afflict other workspaces. Did something change with how `avoidStruts` works? The issue persists even if I reload the configuration, and remains on the first workspace (even if I reload on a different workspace). This is the config I use: [xmonad.hs](https://github.com/Rufflewind/config/blob/9cb0e90214c31c44233371f2e1cba314bfe6f544/home/.xmonad/xmonad.hs). I am on Arch Linux.

### #35 [closed] memory leak on master  (24c)
It consumed (rss) half a gig for me in two days. @bgamari noticed his xmonad consuming 2.5GB. Consumes only 27MB after a restart for me. It may be a result of using ghc-8.0.1.

### #42 [closed] Missing dropdowns in Chromium, unworkable menus in Qt5 apps on dual monitors  (56c)
Hello I'm using Xmonad for a couple of years and at the start of this year I started seeing invisible dropdown in Chromium: [Chromium bug](https://bugs.chromium.org/p/chromium/issues/detail?id=515451) but it seems that it isn't just Chromium problem. At the start Dropdown always worked on one screen and not on the other but now sometimes it doesn't show on both and it is annoying. But bigger problems are Qt5 apps like qtcreator and calibre since non of the menus work or right clicks or tab completitions. (seems that floating windows are a problem) All of this things work in Fluxbox so it seems that Xmonad is the reason. EDIT: none of the problem appear when faking multiple screens with XMonad.Layout.LayoutScreens. I'm using Xmonad 0.12 Xorg-server 1.18.4-1 Arch Linux with: 4.6.4-1-ARCH on i5-3570K with integrated graphic card with mesa: 12.0.1-5 and xf86-video-intel …

### #47 [closed] Uneven monitor hotplugging  (6c)
To reproduce (probably don't need the exact hardware, just the primary monitor at nonzero y offset): 1. Have setup with 1366x768 laptop monitor at (0,256) and 1280x1024 VGA monitor at (1366,0). 2. Have udev rule to run `xrandr --output VGA1 --off` when monitor is disconnected and `xrandr --output LVDS1 --auto --pos "0x256" --output VGA1 --auto --pos "1366x0"` when monitor is connected. 3. Start with monitor plugged in. Spawn and close some windows. 4. Unplug monitor. Spawn and close some windows. Expected behavior: Windows spawn at (0,256) (filling the laptop screen) when monitor is plugged in, and (0,0) (filling the laptop screen) when monitor is unplugged. Observed behavior: Windows spawn at (0,256) (filling the laptop screen) when monitor is plugged in, and (0,256) (leaving the top 256 rows blank) when monitor is unplugged. Persists until `xmonad --restart`.

### #60 [closed] Xmonad compatibility with RandR >= 1.4 and dual-GPUs in multihead  (21c)
I was using xmonad in a very satisfying fashion until the fateful day when I upgraded the nvidia drivers on my Ubuntu laptop, powered by an Optimus enabled GPU. My typical workflow was to: - Set the window arrangement with `xrandr`, typically along the lines of `xrandr --output eDP --auto --output DP-0 --right-of eDP`. Note that the integrated graphics is connected to the Intel chip while the external ports are connected to the discrete NVIDIA card. - Start xmonad with the appropriate number of screens (workspaces = withScreens 2 $ map show [1..9]). It worked very well. With the latest update (nvidia-331 => nvidia-367, I also checked a few in between with the same results), xrandr succeeds without problems in turning on both displays but xmonad only uses one screen, in my case the external display: - xmobar only appears on that screen - It is impossible to move a window to/from a …

### #70 [closed] someone missed a `getWindowAttributes` somewhere  (28c)
so our recent crash reports on Arch have been identified, and they're what I suspected/feared > [16 00:36:20] <coltfred__> I just installed xmonad on a fresh arch. I've used it before on ubuntu and in general everything is basically working. However, when I try and start hexchat (only in xmonad) or if I try and kill stalonetray I get an error in my .xsession-error file about "user error (Error in function getWindowAttributes)". If I start in awesomewm or something else I don't have problems launching the application. not clear *where* this is happening as yet though

### #86 [closed] Serialized state is persistent after reboot  (17c)
Because the state of xmonad is now serialized to a file, my xmonad breaks sometimes because it found the serialized state from the previous session and i end up having to remove the file and restart xmonad. Maybe it's a good idea to remove the statefile after it has been read? Or put it somewhere like /run/user/$(id -u) or /tmp?

### #90 [closed] Workspace/window size reduced to smallest screen size  (12c)
### Problem Description Issue was not present in xmonad < 0.13 Workspace/windows size is reduced to the smallest screen size. In my configuration * 2 x screens 2560x1440 * 1 x laptop screen 1920x1080 - **DISABLED** So in my case when I'm opening new app its size is limited to 1920x1080 ### Configuration File This issue is not configuration specific ![screen_2017-03-10-22-47-07](https://cloud.githubusercontent.com/assets/619794/23815253/a79217b0-05e7-11e7-82c6-796055e7f4cb.png)

### #104 [closed] focusFollowsMouse and clickJustFocuses can be confusing to new users and may not be very ergonomic.  (9c)
### Problem Description Now, I see that the default `clickJustFocuses=True` can serve as a safety device against another default, `focusFollowsMouse=True`. However, `clickJustFocuses=True` often turns a single click into double clicks. If it happens often, it becomes frustrating. `focusFollowsMouse=True` doesn't apply to empty screens. When I move mouse cursor to an empty screen, the screen doesn't get the focus until I click it. I prefer `focusFollowsMouse=True` and `clickJustFocuses=False`. If `focusFollowsMouse=True` was also applied to empty screens, it'd be better. Others might have other preferences, and I think the ergonomics can improve with regard to `clickJustFocuses` and `focusFollowsMouse`.

### #105 [closed] New floating windows sometimes are created partially off-screen  (6c)
### Problem Description New floating windows sometimes are created partially off-screen ### Configuration File Please include the smallest configuration file that reproduces the problem you are experiencing: ```haskell -- -- xmonad example config file. -- -- A template showing all available configuration hooks, -- and how to override the defaults in your own xmonad.hs conf file. -- -- Normally, you'd only override those defaults you care about. -- import XMonad import Data.Monoid import System.Exit import qualified XMonad.StackSet as W import qualified Data.Map as M import XMonad.Util.Run(spawnPipe) import XMonad.Util.Run(safeSpawnProg) import XMonad.Hooks.ManageDocks import XMonad.Hooks.DynamicLog import System.IO import XMonad.Layout.SimpleFloat import XMonad.Layout.Renamed as R import XMonad.Hooks.Place import XMonad.Actions.NoBorders -- The preferred terminal program, which is used …

### #106 [closed] New floating windows are sometimes created under existing floating windows  (6c)
### Problem Description New floating windows are sometimes created under existing floating windows ### Configuration File Please include the smallest configuration file that reproduces the problem you are experiencing: ```haskell -- -- xmonad example config file. -- -- A template showing all available configuration hooks, -- and how to override the defaults in your own xmonad.hs conf file. -- -- Normally, you'd only override those defaults you care about. -- import XMonad import Data.Monoid import System.Exit import qualified XMonad.StackSet as W import qualified Data.Map as M import XMonad.Util.Run(spawnPipe) import XMonad.Util.Run(safeSpawnProg) import XMonad.Hooks.ManageDocks import XMonad.Hooks.DynamicLog import System.IO import XMonad.Layout.SimpleFloat import XMonad.Layout.Renamed as R import XMonad.Hooks.Place import XMonad.Actions.NoBorders -- The preferred terminal program, …

### #116 [closed] XMonad freezes if I choose another window while XMonad prompt is waiting for an input.  (11c)
### Problem Description I noticed this problem while I was using `XMonad.Prompt.ConfirmPrompt`. While `XMonad.Prompt.ConfirmPrompt` is waiting for an input, if I try to give focus to another window, XMonad completely freezes. I have to kill xmonad in virtual terminal. Since `focusFollowsMouse = True` in my XMonad environment, moving a mouse cursor while a confirm prompt is waiting can freeze XMonad. I surmise this is a problem of lack of concurrency in `X`. A workaround is to avoid `X` and use `xfork` and `MonadIO m => m`. This is possible by replacing `XMonad.Prompt.ConfirmPrompt` with my own implementation of dmenu prompt.

### #117 [open] Race?  (13c)
### Problem Description I start xmonad with a .xinitrc file that starts firefox and rox as background processes before starting xmonad, which creates a bar with xmobar. Sometimes, not always, the firefox and rox windows cover the bar, despite the use of avoidStruts. ### Configuration File Please include the smallest configuration file that reproduces the problem you are experiencing: .xinitrc: #!/bin/sh xsetroot -gray rox& firefox& xmonad xmonad.hs: ```haskell -- -- xmonad example config file. -- -- A template showing all available configuration hooks, -- and how to override the defaults in your own xmonad.hs conf file. -- -- Normally, you'd only override those defaults you care about. -- import XMonad import Data.Monoid import System.Exit import qualified XMonad.StackSet as W import qualified Data.Map as M import XMonad.Util.Run(spawnPipe) import XMonad.Util.Run(safeSpawnProg) import …

### #83 [closed] xmonad eats my panel  (17c)
### Problem Description I am running fedora 25 with "xmonad-mate" configuration. I (want to) have 2 mate panels on the screen (as sidebars), showing statuses and application icons. With xmonad-0.13 (built with ghc-8.0.2 via cabal install), these panels are invisible when there is some window on the screen, and they only show up if the screen is empty (no windows) ### Configuration File ```haskell import System.Posix.Env (getEnv) import Data.Maybe (maybe) import XMonad import XMonad.Config.Desktop import XMonad.Config.Gnome import XMonad.Config.Kde import XMonad.Config.Xfce import XMonad.Hooks.SetWMName import MyEventHook main = do session <- getEnv "DESKTOP_SESSION" xmonad $ ( maybe desktopConfig desktop session ) { modMask = mod4Mask , terminal = "gnome-terminal" -- , startupHook = setWMName "LG3D" , handleEventHook = fullscreenEventHook' } desktop "gnome" = gnomeConfig desktop "kde" …

### #80 [closed] Honor fixed size floating windows.  (3c)
### Problem Description If a window has both the minimum and maximum size set in the WM_NORMAL_HINTS property, XMonad will automatically float this window, but you can still change the window size via the hotkeys. Is there any way to enforce this property? ### Configuration File Please include the smallest configuration file that reproduces the problem you are experiencing: ```haskell module Main (main) where import XMonad main :: IO () main = xmonad def ```

### #60 [closed] Xmonad compatibility with RandR >= 1.4 and dual-GPUs in multihead  (21c)
I was using xmonad in a very satisfying fashion until the fateful day when I upgraded the nvidia drivers on my Ubuntu laptop, powered by an Optimus enabled GPU. My typical workflow was to: - Set the window arrangement with `xrandr`, typically along the lines of `xrandr --output eDP --auto --output DP-0 --right-of eDP`. Note that the integrated graphics is connected to the Intel chip while the external ports are connected to the discrete NVIDIA card. - Start xmonad with the appropriate number of screens (workspaces = withScreens 2 $ map show [1..9]). It worked very well. With the latest update (nvidia-331 => nvidia-367, I also checked a few in between with the same results), xrandr succeeds without problems in turning on both displays but xmonad only uses one screen, in my case the external display: - xmobar only appears on that screen - It is impossible to move a window to/from a …

### #61 [closed] Support for custom config locations and recompile strategies  (13c)
I believe that the two topics mentioned in the subject of this issue are related and can be solved together. I'm bringing in everyone from issues #8, #7, #56, and #53. Sorry for the spam if you are no longer interested in these issues. Broadly speaking, there are two main groups of xmonad users: 1. Those who treat xmonad as a window manager with a configuration file. Typically these users have xmonad started by a login manager. In turn, xmonad compiles the configuration file as necessary and then uses `exec` to replace itself with the binary generated by compiling the configuration file. Some of these users are happy to keep their config files in `~/.xmonad`, others would prefer the XDG config directory. 2. Those who treat xmonad as a window manager construction kit. Instead of thinking about their `xmonad.hs` file as a configuration file, these users build up a set of files, possibly …

### #73 [closed] Java GUI problem  (5c)
Java GUI applications only give me a blank window, I tried: main = do xmonad $ defaultConfig { modMask = mod4Mask , startupHook = setWMName "LG3D" -- other customizations } and setting this: _JAVA_AWT_WM_NONREPARENTING=1 and this: AWT_TOOLKIT=MToolkit and I tried using "wmname" suckless tools. None of those methods worked for me. Two questions: 1. Is there any other possibility? 2. What am I getting wrong? I use java 8 and below is my currently minimal xmonad config. import XMonad import XMonad.Hooks.DynamicLog import XMonad.Hooks.SetWMName import XMonad.Hooks.ManageDocks import XMonad.Hooks.FadeInactive import XMonad.Util.Run(spawnPipe) import XMonad.Util.EZConfig(additionalKeys) import XMonad.Actions.UpdatePointer import XMonad.Hooks.EwmhDesktops import System.IO term = "termite" myWorkspaces = ["1","2","3","4","5"] myLogHook :: X () myLogHook = fadeInactiveLogHook fadeAmount where …

### #81 [closed] Add option to create borders externally  (9c)
### Problem Description Borders are created internally, so if borderWidth = 2, a call to xcb_create_window() with width=1280 and height=720 and borderwidth = 0 ends up creating a client area measuring 1276 x 716. For graphics applications that require a fixed window size, this can be particularly annoying if you have to wait for the window to become exposed to resize the window. An .xinitrc file consisting of precisely ``` #! /bin/bash exec xterm ``` can create borders that go outward [like so](http://i.imgur.com/Ycs51pV.png), confirmed via xcb_get_geometry having the correct client area. ### Configuration File ```haskell module Main (main) where import XMonad main :: IO () main = xmonad def ``` ### Checklist - [x] I've read [CONTRIBUTING.md](https://github.com/xmonad/xmonad/blob/master/CONTRIBUTING.md) - [x] I tested my configuration with [xmonad- …

### #103 [closed] Allow single click to focus and pass click to window by default  (4c)
### Problem Description When I move the mouse cursor to a different window, the window gets focus. However, if I press Mod+j and move the mouse cursor in the previously focused window, the focus remains on the newly focused window. Thus, a mouse cursor ends up hovering on an inactive window. While a mouse cursor hovers on an active window, if I click, the window gets focus, but doesn't receive the click event. Thus, I very often have to click twice. This is confusing and frustrating. I searched the internet and found https://mail.haskell.org/pipermail/xmonad/2012-January/012178.html It seems this was not merged since I encountered the same issue again.

### #108 [closed] focus window under mouse pointer  (8c)
In my xmonad config, I have set `focusFollowsMouse=True` so that a window is focused when the mouse pointer is moved onto it. However, I would also like a window to become focused when the mouse pointer is _already_ on it, even if it did not move. Can this be acheived? ### Configuration File ```haskell module Main (main) where import XMonad main :: IO () main = xmonad (def {focusFollowsMouse = True}) ```

### #112 [open] Provide a way to execute some code after all pending XEvents  (9c)
I have some code that allows me to select a chrome tab using rofi or dmenu to be split out of its current window and into a new window. Because of the way that XMonad handles Xevents, the windowset will not reflect the fact that this new window is now focused until after the block finishes executing, and Xmonad has the opportunity to handle the MapRequestEvent associated with the new window. I have some hooks that I would like to run given that I have run a command that should focus a new window (ex: I would like to replace the previously focused window with the newly focused window so I need to minimize the previously focused window). Ideally, there would be some way to execute some X (the XMonad type) action after all pending X events have cleared. This could be accomplished simply by just enqueing a new X event with some id that could be used to recover the X action to be activated.


## herbstluftwm

### #33 [closed] Focus stealing bug  (5c)
Sometimes, dwb or mpv steal the focus, and switching keyboard focus does not work (immediately focuses back to dwb/mpv again). Maybe related (?): Some clients (steam) create windows which have focus without having the focus borde

### #10 [closed] Use xrandr instead of xinerama to detect screens  (1c)
xrandr is probably more standard, but it seems rather hard to use it... Some pointers: - https://github.com/LemonBoy/bar/blob/master/lemonbar.c#L834-L926 - https://github.com/Guff/guffml/blob/3272817b4e659710f3eaa7f3363284b0df105347/guffml/linux/guffml-videomodex11.c - https://git.gnome.org/browse/gnome-panel/tree/gnome-panel/panel-multiscreen.c Also it'd be nice if the monitors get auto-named based on their connecors (`vga1`, `lvds1`, etc.)

### #31 [open] Better multi-monitor tag switching  (2c)
Some people find tag switching with multiple monitors confusing. Some ideas: - if tag X is shown on any monitor, focus it. If not, show tag on current monitor. (bl1nk in IRC) - mimic i3 behaviour -- lock a set of tags on a monitor, and then they always stay there (Rasi)

### #34 [open] Window contents move when focused  (1c)
When focusing a window, the contents briefly move their positions and then move back which is a bit irritating. It's as if the thicker border caused the window to get a wrong size/position for a very short time.

### #35 [closed] Title doesn't get updated correctly with Qt applications  (1c)
When a Qt application changes its window title, somehow hlwm doesn't recognize the title has changed. I can reproduce this with PyQt5 (qutebrowser) and Qt4 (otter browser) applications. PyQt example: ``` import sys from PyQt5.QtCore import QTimer from PyQt5.QtWidgets import QApplication, QWidget app = QApplication(sys.argv) w = QWidget() w.setWindowTitle("Hello") w.show() QTimer.singleShot(1000, lambda: w.setWindowTitle("World")) app.exec_() ``` After a second, the title changes from "Hello" to "World". What I found out so far: - `xprop | grep '^_NET_WM_NAME'` shows the updated title - The `window_title_changed` hook is never emitted. - When I unfocus the window and focus it again, `focus_changed` is emitted with the old title ("Hello") - When I switch tags and switch back, the title is then updated (I get `focus_changed` with "World" when focusing it)

### #54 [closed] Spurious focus for client under mouse cursor  (5c)
Whenever the focus is changed, the client under the mouse cursor temporarily gets focus. That is, it is sent focus in and focus out events. This seems to happen on all focus changes, even if they happen in other frames etc. It is easy to test this behavior with an application that reacts to focus (e.g. a terminal that changes the cursor shape) or xev.

### #64 [closed] XFetchName doesn't always return a name.  (7c)
Even though a window might have a WM_NAME property, XFetchName doesn't always return a name - for some reason. Googling around a bit I couldn't figure out why though. I noticed that herbstluftwm didn't name Steam's windows. Steam windows don't have a _NET_WM_NAME property, so herbstluftwm falls back to calling XFetchName to get the WM_NAME property. Running herbstluftwm in verbose, it prints that "no title for window xxx found, using ''" when a Steam window is focused, where xxx is the ID of a Steam window. This means that XFetchName returned 0, even though using xprop shows the window to indeed have a WM_NAME property. I did a test of the XGetWMName function, using code from this stack overflow thread [1], which returned the correct name, where XFetchName returned no name. I propose to change XFetchName to XGetWMName. [1] https://stackoverflow.com/questions/8825661/xfetchname-always- …

### #73 [closed] Window placement in floating mode  (14c)
Current strategy seems to be to place windows in the top left of the current monitor. While it was the easiest thing in the world to create a hook, and subsequently a script that gets triggered on windows with certain conditions (i.e. no size hints for location), that centres the window, this has a disadvantage. When I do this there is an almost unnoticeable movement before the window moves to the centre of the monitor. With a compton fade effect not so much noticable, without it, it looks quite clunky. Would it be possible (maybe in next release) to implement placement strategy natively in the WM ? Possible values could be: - Centre (or center, I never know.. [[1]](http://grammarist.com/spelling/center-centre/)) - Corner (all of them) - Smart (I never use this, and never will :) )

### #80 [closed] Crash with latest commit  (3c)
Hello, the latest commit (https://github.com/herbstluftwm/herbstluftwm/commit/0e370d5ea3ccd73a) produces crashes if I try to do any of the following: 1) switch to a tab with a window 2) open a window on an empty tab resetting to before that commit hlwm works just fine. Any advice on how I could gather more information to help with this crash? Awesome work you do, btw :-) Been using hlwm for over 2 years now and have never wanted anything else.

### #82 [closed] Mouse resizing in tiling mode crashes  (1c)
In the irc channel, @johnlate reports: - I've just tried e67e229 (but i think the issue could be in ed2cd33): multimonitor, tiling mode, resize-drag with mouse -> segfault - 0x000000000042190f in mouse_function_resize_tiled (me=0x7fffffffdee0) at [src/mouse.cpp:404](https://github.com/herbstluftwm/herbstluftwm/blob/master/src/mouse.cpp#L404) - 404 `if (parent->content.layout.align == ALIGN_HORIZONTAL) {` - Sorry, I don't have time to look at the code right now. However I can't reproduce it yet.

### #91 [closed] pop-ups as floating windows  (8c)
Popups by applications should be automatically floated, and not treated as normal windows with which the originating application shares screen space.

### #94 [closed] "and" command causes X to crash when compiled with musl  (12c)
When `and` is used, X crashes immediately. Seems to be independent of the content of the expression. I tested it with `herbstclient and 1 = 1` to play around with it. I tried to track down where it occured, and I made it this far. ``` --- herbstluftwm/src/command.cpp 2017-02-22 03:07:01.187067000 -0500 +++ herbstluftwm-musl/src/command.cpp 2017-03-04 10:46:11.278686757 -0500 @@ -313,7 +313,9 @@ int status; // TODO why isn't the cast (char** -> const char**) done automtically? if (bind->has_output) { + printf("This line breaks\n"); status = bind->cmd.standard(argc, (const char**)argv, output); + printf("This is never seen\n"); } else { status = bind->cmd.no_output(argc, (const char**)argv); } ```

### #97 [closed] Crash with merge_tag command  (1c)
When calling `herbstclient merge_tag TAG TARGET`, herbstluftwm crashes when TARGET doesn't exist. Example with the default `autostart` file: `herbstclient merge_tag 9 foo`. The crash only happens when TAG is not focused. In that case i get the expected error message: ``` merge_tag: Tag "foo" not found merge_tag: Cannot merge the currently viewed tag ``` (PS: Is there an easy way to enable debug symbols and set optimization to O0 in the build system? I seem to be unable to figure it out ...)

### #103 [closed] The hook consequence fires twice with different window IDs  (2c)
Create a rule with a `hook` consequence. Then for each new window, the rule is fired twice with different window IDs (where one of them is invalid).

### #167 [closed] Segfault in HSFrameLeaf::split()  (5c)
Doing a exploding split with two windows. It turns out that the argument `int childrenLeaving` is -1. This leads to a segfault in line 938: vector<HSClient*> leaves(clients.begin() + childrenStaying, clients.end()); because this goes wrong first: int childrenStaying = std::max((size_t)0, clients.size() - childrenLeaving); childrenStaying is maxed to clients.size() + 1. But why do we get -1 as parameter in the first place? This problem might have been introduced in 056f522cb6d39eaf1d15455454a3e628f62135f8. ``` size_t count1 = 0; if (exploding) { count1 -= (frame->clientCount() + 1) / 2; // new count for the first frame } ``` In the exploding case, count1 appears to always be negative (overflown, that is).

### #194 [closed] (Virtual) desktop size not adjusting upon xrandr resolution change  (2c)
My system starts in the wrong resolution, and I manually adjust it after start. Now it appears as if herbstluftwm still thinks the old resolution is active, and only tiles windows on the upper left rectangle that represented the screen before the resolution change. If I switch to floating mode, I can move windows into the new extra area. But in tiling mode, only the upper left rectangle is considered. To demonstrate, see the images below. This is floating, I can reach all areas: https://imgur.com/8DhtpMq This is tiling, you can see that a new split is in the bottom right of the reduced area: https://imgur.com/kGKROyE This is "fullscreen", again only the reduced are is considered: https://imgur.com/GHn37x8

### #222 [closed] Invalid read in ewmh_init()  (2c)
Running winterbreeze with valgrind gives the following error/warning on startup: ``` ==19722== Memcheck, a memory error detector ==19722== Copyright (C) 2002-2017, and GNU GPL'd, by Julian Seward et al. ==19722== Using Valgrind-3.14.0 and LibVEX; rerun with -h for copyright info ==19722== Command: ./herbstluftwm --verbose -c ../share/autostart ==19722== ==19722== Invalid read of size 8 ==19722== at 0x4AEBA30: _XData32 (in /usr/lib/libX11.so.6.3.0) ==19722== by 0x4AC5D99: XChangeProperty (in /usr/lib/libX11.so.6.3.0) ==19722== by 0x150215: ewmh_init() (in /home/thorsten/git/winterbreeze/build/herbstluftwm) ==19722== by 0x11F282: main (in /home/thorsten/git/winterbreeze/build/herbstluftwm) ==19722== Address 0x5b5a708 is 0 bytes after a block of size 8 alloc'd ==19722== at 0x4837DEF: operator new(unsigned long) (vg_replace_malloc.c:334) ==19722== by 0x1501DD: ewmh_init() (in …

### #402 [closed] [Question] how to fix background trailis  (5c)
If I have a floating window, whatever is "behind it" never refreshes. So if I get a notification that will come on that area, or I refresh size of that floated window parts of that box stay in the "background" behind it. Can I fix it somehow please? Thanks!

### #85 [open] Binding `Shift+a` isn't the same as binding `A`  (5c)
This is slightly confusing, especially since the man page recommends to read `keysymdef.h` for reference -- which has a different keysym for `a` and `A`. I suppose if I had a keyboard with a dedicated `A` key, HLWM might pick that up, but I don't know if such a keyboard exists. This of course applies to any other key, like `Shift+1` not being the same as `exclam` and so on. I'd like to give a detailed suggestion, but I don't really know the implementation details here. However, I do know that `xev` reports receiving a `!` keysym when I do `Shift+1`. Perhaps the same logic could be used in HLWM, to more closely follow the principle of least astonishment. Cheers

### #95 [closed] Conceptual similarities between attributes and settings. Why do we have both?  (6c)
It bugs me a little that we have both attributes and settings, where attributes are just like settings, except with more features. Why not move all settings (`herbstclient set ...`) into an attribute class, say, an `hlwm` class. So `hc set default_frame_layout 1` would become `hc attr hlwm.default_frame_layout 1`. All the `settings` code could be deleted, and the man page would get shorter. What's the reasoning behind having both?

### #99 [closed] Balance frames? [feature request]  (3c)
Is there a way to reset the frame split fractions? (as in say to 0.5 for default, or to a given fraction)

### #113 [closed] X11/extensions/shape.h issues  (3c)
Hello, I am trying to install herbstluftwm on the latest Solus and have run into an issue with shape.h **X11/extensions/shape.h** does not exist in my directories, but **xcb/shape.h** does. However, when I redirect **x11-utils.cpp** to **xcb/shape.h** it throws a bunch of errors like so src/x11-utils.cpp: In function ‘void window_cut_rect_hole(Window, int, int, int)’: src/x11-utils.cpp:41:31: error: ‘ShapeBounding’ was not declared in this scope XShapeCombineMask(d, win, ShapeBounding, -bw, -bw, p, ShapeSet); ^~~~~~~~~~~~~ src/x11-utils.cpp:41:59: error: ‘ShapeSet’ was not declared in this scope XShapeCombineMask(d, win, ShapeBounding, -bw, -bw, p, ShapeSet); ^~~~~~~~ src/x11-utils.cpp:41:5: error: ‘XShapeCombineMask’ was not declared in this scope XShapeCombineMask(d, win, ShapeBounding, -bw, -bw, p, ShapeSet); ^~~~~~~~~~~~~~~~~ src/x11-utils.cpp:41:5: note: suggested alternative: …

### #167 [closed] Segfault in HSFrameLeaf::split()  (5c)
Doing a exploding split with two windows. It turns out that the argument `int childrenLeaving` is -1. This leads to a segfault in line 938: vector<HSClient*> leaves(clients.begin() + childrenStaying, clients.end()); because this goes wrong first: int childrenStaying = std::max((size_t)0, clients.size() - childrenLeaving); childrenStaying is maxed to clients.size() + 1. But why do we get -1 as parameter in the first place? This problem might have been introduced in 056f522cb6d39eaf1d15455454a3e628f62135f8. ``` size_t count1 = 0; if (exploding) { count1 -= (frame->clientCount() + 1) / 2; // new count for the first frame } ``` In the exploding case, count1 appears to always be negative (overflown, that is).

### #222 [closed] Invalid read in ewmh_init()  (2c)
Running winterbreeze with valgrind gives the following error/warning on startup: ``` ==19722== Memcheck, a memory error detector ==19722== Copyright (C) 2002-2017, and GNU GPL'd, by Julian Seward et al. ==19722== Using Valgrind-3.14.0 and LibVEX; rerun with -h for copyright info ==19722== Command: ./herbstluftwm --verbose -c ../share/autostart ==19722== ==19722== Invalid read of size 8 ==19722== at 0x4AEBA30: _XData32 (in /usr/lib/libX11.so.6.3.0) ==19722== by 0x4AC5D99: XChangeProperty (in /usr/lib/libX11.so.6.3.0) ==19722== by 0x150215: ewmh_init() (in /home/thorsten/git/winterbreeze/build/herbstluftwm) ==19722== by 0x11F282: main (in /home/thorsten/git/winterbreeze/build/herbstluftwm) ==19722== Address 0x5b5a708 is 0 bytes after a block of size 8 alloc'd ==19722== at 0x4837DEF: operator new(unsigned long) (vg_replace_malloc.c:334) ==19722== by 0x1501DD: ewmh_init() (in …

### #20 [open] Click-focus for empty frames  (3c)
We should be able to focus empty frames with click, or with the mouse if focus_follows_mouse is on

### #21 [open] Window size hint handling with cell-windows (e.g. terminal)  (2c)
Some windows really want to have a size which is a multiple of a given cell size (e.g. terminal). We probably should give them the size they want and just display a gap for the rest.

### #42 [open] Feature request: Client size exposure/handling  (1c)
It is not possible to set the window size for a pseudotile window. A current workaround is to switch to floating mode, resize the window, switch back. In winterbreeze we should add width, height attributes that are exposed and can be set via object tree. Setting them needs to trigger the resp. updates. A typical use-case is file open dialogs. Dialogs are pseudotile in many configurations, and the default dialog window size is unnecessary small. Blowing it up to the whole frame might also be inconvenient. Another example is a GUI calculator. As a bonus, we could implement increment/decrement operation to the integer attribute setter. E.g. setting the attribute width to "+10" increases the window size by 10 pixels. However, this would not work when negative values are allowed.

### #86 [open] Tag Switching on Lock  (2c)
Previously when Herbstluftwm was locked via herbstclient lock, rendering switching between tags was also disabled. This allowed for the scripting of some pretty interesting behaviour for managing tags and frames that were "out of view" while never rendering the tag that was being managed. Is the change to the new behaviour intentional? Or should I expect lock to continue to lock only drawing within tags, rather than switching between them?


## qtile

### #227 [closed] Intellij Idea communitiy edition, text input does not work  (28c)
I am unable to edit any text field, either main text or option fields. Drop down menus, project file browser and other elements work as expected. If i click on a line it actually gets selected, and it seems like cursor gets installed in the click location (as evident from variable highlighting), but cursor symbol is not displayed and i am unable to input or delete any text. I am using qtile 0.5 and intellij idea 11.1 on openjdk7 on arch linux. wmname is set to LG3D. Everything works fine on awesome wm, so it is not openjdk vs sun jdk issue.

### #316 [closed] Cannot open new X applications and then crash after resume from hibernate  (20c)
If i put my computer in hibernate mode for some time (more than 20 minutes at least), qtile behaves strangely after resuming my session. First I cannot open any new X applications, for instance for xterm: ``` ➜ ~ xterm No protocol specified xterm: Xt error: Can't open display: :0 ``` Then qtile crashes if i reload my configuration. I'm using qtile 0.6 with latest updated archlinux.

### #354 [closed] Post sleep pausing  (24c)
As alluded to in issue #349, sometimes when a machine running Qtile develop (in this specific case, I'm on 8f40c608) wakes up it typically takes QTile 30-60 seconds to become responsive. The focused application is responsive and it seems keyboard commands are buffered and sent to Qtile when roused. This wasn't happening to me until I upgraded Ubuntu to 13.10.

### #379 [closed] CPU usage leak  (14c)
Running qtile master. Over a timespan of about 24 hours, qtile's CPU usage rises from trivially low to nearly 100%. In the worst stages, qtile is non-responsive to keyboard shortcuts, forcing me to kill it externally.

### #275 [closed] Flickering with Spotify  (22c)
When I open spotify, it is bigger than the screen size and have a flickering on some portion of the interface. It's not very noticeable, but it's annoying. I'm using Ubuntu 12.04 and Qtile v5.0 from PPA repository. This is my config: http://pastebin.com/rkbu37pz I tried different layouts and there's no difference. I haven't noticed this in any other application. It works OK in a different WM (awesome).

### #141 [closed] Google Chrome window flickers when link is opened in a new tab  (15c)
When I open a link in Google Chrome or Chromium using a middle mouse click or by selecting "Open Link In New Tab", the browser's window (the area with a web page and address bar) flickers a bit. It's not very noticeable, but it's annoying. I'm using Ubuntu 12.04 and tried Qtile from it's repository as well as the newest version from Git. I tried Google Chrome, Chromium from repositories, latest Chromium dev. build and it's always the same. This is my config (I disabled mouse dragging): http://pastebin.com/98fDHj03 I tried different layouts, disabling or enabling bar and there's no difference. I haven't noticed this in any other application. It works OK in a different WM (XMonad).

### #417 [closed] Firefox window is not redrawn after returning from floating layout  (22c)
When Firefox is toggled to floating layout and back, the window stops being redrawn. You can click the browser tabs and the window title in the tasklist widget does update, but the window does not. Moving the window to a different group "fixes" this. This does not appear to be related to #416 as the issue existed before it was merged. I'm not sure if this is exclusive to Firefox, but it's the only application where I've seen this issue. ``` bash [develop] erik@saltmine:~/abs/qtile-git/qtile% git describe v0.1-1413-g1b46a8b ```

### #414 [closed] qtile incorrectly reserving space for wkline toolbar  (15c)
I've been trying to use [wkline](https://github.com/Lokaltog/wkline/) with Qtile today, and I've encountered a strange behaviour. It appears that Qtile is trying to reserve space for the wkline bar, but locating the bar incorrectly. I believe this relates to #385. The result of this is that the bar appears over existing Qtile windows, and said windows are shifted up by the number of px that wkline has asked to be reserved. This is more easily explained in pictures. Top bar overlays existing Qtile windows: ![top-wkline-bar](https://cloud.githubusercontent.com/assets/1651943/2812133/9b38481a-ce43-11e3-99d4-659b543abb55.png) Bottoms of Qtile windows are shifted up by the quantity of reserved space requested by wkline: ![2014-04-27-153149_1919x160_scrot](https://cloud.githubusercontent.com/assets/1651943/2812135/c8dd5936-ce43-11e3-8868-b95f41cde643.png) I am not sure if this is a …

### #35 [closed] systray is empty upon restart of qtile  (13c)
If I restart qtile (say after plugging in an external monitor) while there are items in the systray widget, upon restart the systray is empty.

### #39 [closed] Memory being gobbled up  (12c)
I've been running qtile for about 2 weeks now as my primary wm. I was running some other stuff and started swapping today and just noticed that the qtile process was taking up 700 Megs of res. mem. This is probably after running for 3-4 days. I haven't looked into it at all, just thought I'd file a bug for now.

### #68 [open] Only one window should have focus decoration with multi screens  (11c)
When switching focus from one screen to another there ends up being a focused window on each screen (ie both windows have the focus decoration). I would think that since only one window can have focus, there should only be one window with focus decoration.

### #32 [closed] external (xrandr) monitor support borked  (9c)
Am trying out qtile as primary wm now on my laptop. When I turn on my external monitor everything (both screens) go to that window. I use the following command: xrandr --output VGA1 --auto xrandr --output LVDS1 --right-of VGA1 --auto It seems like an x offset is not correct somewhere.... ps - I thought I had this bug filed before, but can't find it

### #338 [closed] Crash when switching from Chrome full-screen using Slice layout  (8c)
Hi, while in the Slice layout, if you switch to full-screen in Chrome (F11) and then switch back, Qtile will crash with the following stack trace: ``` 2013-08-19 23:05:52,527 ERROR _xpoll:568 Got an exception in poll loop Traceback (most recent call last): File "/usr/lib/python2.7/site-packages/libqtile/manager.py", line 564, in _xpoll r = h(e) File "/usr/lib/python2.7/site-packages/libqtile/window.py", line 1055, in handle_ClientMessage self.fullscreen = (fullscreen_atom in current_state) File "/usr/lib/python2.7/site-packages/libqtile/window.py", line 709, in fullscreen self.disablefloating() File "/usr/lib/python2.7/site-packages/libqtile/window.py", line 881, in disablefloating self.group.mark_floating(self, False) File "/usr/lib/python2.7/site-packages/libqtile/group.py", line 226, in mark_floating i.focus(win) File "/usr/lib/python2.7/site-packages/libqtile/layout/max.py", line …

### #320 [closed] Multiscreen crash at startup  (8c)
Qtile is working perfectly when I use it on one screen but when I dock my laptop to its portable media dock (it basically adds a new graphic card). I have this error at startup : ``` Traceback (most recent call last): File "/usr/bin/qtile-python2.7", line 89, in <module> q = make_qtile() File "/usr/bin/qtile-python2.7", line 83, in make_qtile state=options.state, File "/usr/lib64/python2.7/site-packages/libqtile/manager.py", line 141, in __init__ self._process_screens() File "/usr/lib64/python2.7/site-packages/libqtile/manager.py", line 219, in _process_screens self.groups[i], File "/usr/lib64/python2.7/site-packages/libqtile/config.py", line 135, in _configure i._configure(qtile, self) File "/usr/lib64/python2.7/site-packages/libqtile/bar.py", line 163, in _configure self.opacity File "/usr/lib64/python2.7/site-packages/libqtile/window.py", line 575, in create i = Internal(win, qtile) …

### #377 [closed] Launching applications that require stdin input hangs qtile  (6c)
Some applications like bashbug causes qtile to hangup. And the only way out is to kill X. Happens regardless if the program was launched by qtile or any other app.

### #274 [closed] LC_ALL set to empty string in Ubuntu 12.10 cusing crash at startup  (4c)
On my ubuntu 12.04 **LC_ALL** is unset as it overrides all other locale variables according to man page of locale. $ locale LANG=en_IN LANGUAGE=en_IN:en LC_CTYPE="en_IN" LC_NUMERIC=en_US.UTF-8 LC_TIME=en_US.UTF-8 LC_COLLATE="en_IN" LC_MONETARY=en_US.UTF-8 LC_MESSAGES="en_IN" LC_PAPER=en_US.UTF-8 LC_NAME=en_US.UTF-8 LC_ADDRESS=en_US.UTF-8 LC_TELEPHONE=en_US.UTF-8 LC_MEASUREMENT=en_US.UTF-8 LC_IDENTIFICATION=en_US.UTF-8 **LC_ALL**= $ qtile locale.setlocale(locale.LC_ALL, locale.getdefaultlocale()) is throwing exception as: Traceback (most recent call last): File "/usr/local/bin/qtile", line 5, in <module> pkg_resources.run_script('qtile==0.5', 'qtile') File "/usr/lib/python2.7/dist-packages/pkg_resources.py", line 499, in run_script self.require(requires)[0].run_script(script_name, ns) File "/usr/lib/python2.7/dist-packages/pkg_resources.py", line 1235, in run_script …

### #122 [closed] Prompt widget: New windows should not steal focus from it  (8c)
Also, moving focus explicitly while in the prompt widget should either close it or move focus to that window and not close it but also not lose its focus.

### #265 [closed] Prompt widget doesn't support capslock  (11c)
Shift works, capslock is ignored.

### #125 [closed] Focus does not change when mouse is clicked.  (10c)
Reproduce: - Hover mouse over window and cause window to be focused. - Change focus to another window using a key binding. - Click on the original window - Focus is not changed back to that window.

### #297 [closed] Drag windows within positions of layout using mouse  (10c)
Hi, I used awesome wm before, and there was this feature that I could switch window position by holding Mod + MouseButton1 click and move. For instance, if I have a RadiotTile layout with 6 windows, I can Mod + click1 the 1st window and move it to the 6th position. Right now there is a command set_position_floating that will set the window as floating and remove it from the Tile, but the desired feature is to just move the window position instead of make it float. Thank you.

### #365 [closed] Fixes problems with focus when clicking windows  (4c)
This patch fixes the issue described in the #125. It also automatically focuses window under cursor while moving (mod+mouse1 in default config) instead of moving other window if one is already focused. Adds additional option to bring to front floating windows by clicking them.

### #383 [closed] Pseudoscreens with same x and y share Screen  (9c)
Pseudoscreens with the same x and y coordinate will not be assigned their different screens. Rather, a single screen will be created with the largest width and height of the pseudoscreens sharing the same origin. This resolves issues when cloning displays, as both Screens would occupy the same location, and maneuvering groups in this configuration was tricky. Resolves #378

### #402 [closed] Group custom layouts  (4c)
Implements overriding layouts list for group, allowing such Group declaration: ``` python Group("floating-only", layouts=[layout.Floating()]) ```

### #415 [closed] Max layout: don't just display the active window  (12c)
In the Max layout, when using an application that is not floating, but has alpha transparency, the wallpaper/background is shown instead of the application under it in the stack. This appears to be so because only the active window is displayed in the Max layout, and thus there is no application under it to _be_ displayed. I often use a transparent terminal, and like being able to see through to the application "underneath", for instance a web browser. It would be nice to be able to do this with qtile.


## spectrwm

### #21 [closed] spectrwm doesn't start with special setxkbmap settings in .xinitrc  (2c)
**EDIT** i also tried with cwm and dwm; they work hi, i'm using spectrwm 2.3 on openbsd-current (snapshot from nov. 3rd). spectrwm gets started by startx, thus reading my .xinitrc. That usually works fine, until i change my keyboard settings to the german neo-variant in .xinitrc before starting spectrwm: ``` in .xinitrc: setxkbmap -layout de -variant neo exec /usr/local/bin/spectrwm ``` It fails every time i call it with neo, even if its just the second layout like here: ``` in .xinitrc: setxkbmap -layout de,de -variant neo,basic -option -option grp:sclk_toggle exec /usr/local/bin/spectrwm ``` If i remove neo or substitute it with e.g. us, it works. I don't know how a keyboard setting could disturb spectrwm, but it crashes and i'm back to the console. However, if i start spectrwm without any setxkbmap command and the execute ``` setxkbmap de neo ``` in a terminal, it works without any …

### #22 [closed] The well known LD_PRELOAD bug when launching an xterm  (5c)
Every time I start an xterm, I get this message multiple times: ``` ERROR: ld.so: object 'libswmhack.so.0.0' from LD_PRELOAD cannot be preloaded: ignored. ``` If I add LD_PRELOAD="" in .bashrc, then: 1) launching an xterm typing the command "xterm" is OK (no error message); 2) launching an xterm with the keyboard shortcut still causes the error message to get displayed (1 time). So far, the workaround is to add "LD_PRELOAD=''; clear" in ~/.bashrc: The error message about LD_PRELOAD is cleared and I do not see it anymore. But it is only a workaround. (Version: 2.3.0)

### #28 [closed] google-chrome always steals focus  (7c)
Even though I have this in ~/.spectrwm.conf: quirk[._:._] = NOFOCUSONMAP ... this doesn't prevent google-chrome from stealing the focus every time it opens a new tab, or when I start it with a spectrwm bind. If I call chrome from terminal, from mutt or wherever, chrome always steals the focus from anything. And it also doesn't matter if it is already running or not. Shouldn't the above quirk prevent this?

### #40 [closed] bind keys not mapped correctly after wm start  (6c)
Spectrwm 2.5.0, Arch linux: $localectl System Locale: LANG=en_US.UTF-8 VC Keymap: dvorak X11 Layout: us,ru X11 Model: pc104 X11 Variant: dvp, In spectrwm.conf I have: bind[menu] = MOD+dollar bind[term] = MOD+Return bind[ws_prev] = MOD+semicolon After wm start MOD+dollar not works but MOD+l (p in qwerty) works. When I press MOD+semicolon (q in qwerty) it looks like original restart command starts. After this all my custom keybindings works fine. Also MOD+return works as well as MOD+Shift+Return, while I only have MOD+Return. My config https://raw.githubusercontent.com/kazufukurou/dotfiles/master/.spectrwm.conf

### #47 [closed] Consuming too much cpu and freezes  (2c)
Hi, I've been using spectrwm for a long time for many releases, and 2.5.0 seems to me like the one that is giving me more problems. Right now, with firefox and tmux open, I'm getting 99% use of CPU. 1465 myuser 20 0 2048m 733m 472 R 99 19.2 33:27.92 spectrwm It's also frozen when I was opening a link with a pdf in firefox (use external program to open the file). Almost always when using vmplayer, spectrwm loses control of keys or mouse and I'm stuck at the screen where vmplayer is. The only thing to do is bring down X. Is it only me with these problems? My version: 2.5.0 running on a Linux Slackware 64 bits 14.1 box.

### #56 [closed] Restarting changes workspaces and reorders windows.  (6c)
When restarting spectrwm, the current workspace is lost (the first is displayed) and the placement of windows in each workspace is seemingly randomized. The first part is a minor inconvenience, but it would be nice if it were fixed to give a seamless restart. The second part for me is a little more annoying because I expect my windows to remain in the same place and have the same size lest their interfaces/scrollbacks change or my muscle memory fail me.

### #62 [closed] Regions Reset on Screen Refresh due to Suspend, TTY Switching  (3c)
On Arch Linux with systemd, my regions are reset upon waking from suspend or switching TTYs. Is this behavior unavoidable due to the display architecture or would a fix be possible? As it is, a quick restart of the window manager repairs it but windows may be garbled due to their change in size.

### #91 [closed] mod with capsLock / caps_toggle  (1c)
hi guys first of all . thanks for spectrwm is amazing very tiny/fast/simple now is time for a question =P how i can change the Mod key? in the manPage i only see 2 option Mod1 [alt] Mod4 [Super] i have an old keyboard [compaq-mx11800] without Mod4 and i use a lot Mod1. is posible to change Mod for capsLock key ??

### #94 [closed] After restart, equal sign in a binding doesn't work  (12c)
Strangest thing; on a US keyboard, these bindings work when starting spectrwm fresh, but after a `restart`, they stop working (ie.: I just get `^[+` in the terminal window, if I wanted to resize that) `bind[height_grow] = MOD+Shift+equal` `bind[width_grow] = MOD+equal` Uhm, spectrwm is the master branch from git (ee4e60a8d), but I don't really know what other software components' version numbers are informative, so I'll just randomly put something here: xorg-server-1.16.4 xf86-input-keyboard-1.8.0 xf86-input-evdev-2.9.2

### #101 [closed] autostart broken in 2.7.0 (Linux)  (2c)
Hi! autostart feature got broken in 2.7.0. I get as much `spectrwm: spawn: execvp: : No such file or directory` lines on the xsession error output as there are autorun entries in my _.spectrwm.conf_. Eg.: ``` autorun = ws[1]:/usr/bin/st ``` Running spectrwm in strace outputs (along with a lot of stuff, of course) these: ``` 30729 execve("/bin/sh", ["sh", "-c", "which /usr/bin/st"], [/* 63 vars */]) = 0 30729 execve("/bin/which", ["which", "/usr/bin/st"], [/* 63 vars */]) = 0 30729 brk(0xcd9000) = 0xcd9000 30729 stat("/usr/bin/st", {st_mode=S_IFREG|0755, st_size=58776, ...}) = 0 30729 getgroups(64, [4, 7, 11, 17, 18, 19, 83, 84, 86, 93, 100, 1000]) = 12 30729 fstat(1, {st_mode=S_IFREG|0600, st_size=1644, ...}) = 0 30729 mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x7fe9b8d01000 30729 write(1, "/usr/bin/st\n", 12) = 12 30729 exit_group(0) = ? 30729 +++ …

### #103 [closed] Broken bindings with NumLock on  (0c)
Hi! Turns out, that this commit 42bccafc30073c1428d63b93d5db8b749491c8d4 (figured out with bisecting) has broken the bindings on Linux when NumLock is turned on. Every key binding works without NumLock, and none of them with NumLock on. I realize that this is a huge diff, so please work with me, and I'll test parts of it, if needed. Thanks, Daniel

### #105 [closed] Configuration issues - unbinding and custom bindings  (2c)
I've just upgraded to v2.7.2 (on arch linux) and all of a sudden my config is giving 11 exceptions, all of them "invalid data for bind". They seem to be associated with: 1. unbinding e.g. bind[] = MOD+Shift+Enter 2. custom bindings e.g. bind[cycle_layout] = MOD+Space Is this a known issue? Is there an available fix?

### #108 [closed] removing external monitor eats windows  (6c)
steps to reproduce: - `exec spectrwm` from `startx` - spawn windows (workspace 1, 2) - setup external monitor: `xrandr --output eDP1 --auto --output DP2 --auto --above eDP1` - unplug external monitor - remove external monitor with `xrandr --auto` - windows are eaten. in my case, windows get eaten from workspace 1. the processes are still running, but the windows are gone and unaccessible (not in m-S-w)

### #120 [closed] regression: iconified windows enter buggy state when plugging a new screen  (3c)
I noticed something new after switching from 2.6.2. Start with one screen (laptop), create a window of sorts (terminal) and iconify it. Plug a new monitor. The iconified window gets de-iconified and starts to flicker like mad, as if the state of the window gets somehow scrambled when a new screen/region is added.

### #124 [closed] LD_PRELOAD is set for all processes launched by spectrwm  (2c)
I noticed when building my own software I had the error: ``` ==27821==ASan runtime does not come first in initial library list; you should either link runtime to your application or manually preload it with LD_PRELOAD. /bin/sh: line 1: 27821 Aborted (core dumped) ``` This was caused by spectrwm setting the `LD_PRELOAD` for all sub-processes, including the terminal launched.

### #127 [closed] Window flickering issue when reconfiguring screen outputs  (13c)
I don't how put my finger on this, but here's the deal: Given are three screens: ``` Screen 0: minimum 8 x 8, current 3840 x 1080, maximum 32767 x 32767 eDP1 connected 1600x900+1920+0 (normal left inverted right x axis y axis) 309mm x 174mm 1600x900 60.04*+ 40.03 1368x768 60.00 1280x720 60.00 1024x768 60.00 1024x576 60.00 960x540 60.00 800x600 60.32 56.25 864x486 60.00 800x450 60.00 640x480 59.94 720x405 60.00 640x360 60.00 DP1 disconnected (normal left inverted right x axis y axis) DP1-1 connected 1920x1080+0+0 (normal left inverted right x axis y axis) 509mm x 286mm 1920x1080 60.00*+ 1600x900 60.00 1280x1024 75.02 60.02 1152x864 75.00 1024x768 75.08 60.00 800x600 75.00 60.32 640x480 75.00 60.00 720x400 70.08 DP1-2 connected 1920x1080+1920+0 (normal left inverted right x axis y axis) 509mm x 286mm 1920x1080 60.00*+ 1600x900 60.00 1280x1024 75.02 60.02 1152x864 75.00 1024x768 75.08 …

### #128 [closed] Dvorak layout not recognized at startup  (13c)
Openbsd 5.9 -stable, Spectrwm 2.7.2 When Spectrwm starts up the first time it doesn't recognize that I'm using the Dvorak layout. A restart is required for it to function properly.

### #132 [closed] Button tooltips appearing on the "other" screen  (5c)
I was always wondering if this is a LibreOffice or Spectrwm issue. I have two screens next to each other (left and right), and when I have a LO window open, and hover the pointer over a button, the tooltip appears on the other screen's closest edge (but vertically aligned with the button). ![tooltip](https://cloud.githubusercontent.com/assets/1006712/15498270/41fcdfca-219f-11e6-877f-ca2cfd3d43fc.png)

### #117 [closed] execvp: No such file or directory  (1c)
When I press M-s (screenshot_all), no screenshot appears to be saved, and I get the message `spectrwm: spawn: execvp: No such file or directory`. The problem occurs in spectrwm V2.7.2, both on Fedora 22 and on OpenBSD 5.8-STABLE.

### #130 [closed] regression: iconified windows enter buggy state when plugging a new screen  (4c)
So, it seems that #120 didn't fix the issue completely indeed. With linux + xorg + intel driver, when I plug a new screen, the iconified windows still enter a buggy state. They don't flicker anymore (the fix for #127 seems to have resolved this), but they still show up uniconified (and unmanaged - they don't have a border and/or respond to focus cycling). Indeed, I can "uniconify" the window to fix the window state. Of note is the fact that when I plug the new screen I also set it as the new primary. I cannot _always_ reproduce the issue, unfortunately.

### #135 [closed] autorun fails placing steam  (2c)
i have `autorun = ws[2]:steam` in my config, but steam shows up in workspace 1. using `quirk[Steam] = WS[2]` works fine.

### #89 [closed] System tray  (3c)
I would like the spectrwm bar to have a system tray. I may look into this one day -- it's the only thing that prevents me from using spectrwm at the moment. We may be able to steal code from i3 for example.

### #57 [closed] Request: Pointer follows Focus  (6c)
This is a feature I grew quite accustomed to in xmonad. Essentially, when window (or screen) focus is changed via the keyboard, the mouse cursor is warped to the location of the focus. In xmonad this was configurable, but a reasonable default IMO is to warp the pointer to the center of the window. This way bumping the mouse/trackpad will not change focus as it would if it were warped to a corner. As it is, the focus follows pointer is annoying when using a combination of keyboard and cursor focus. When focused in a window where the cursor isn't, if you want to change focus to where the cursor IS you must first leave the window and then reënter it. Even if you don't use a hybrid of cursor and keyboard focus selection, changing workspaces (which cannot be done via the cursor) will still usually result in a mismatch between the focus and the cursor position.

### #68 [closed] No warp_pointer update on layout change  (2c)
I use the warp_pointer feature that warps the cursor to the middle of the focused window. I believe this feature is triggered on focus change. It would be nice if it was triggered by layout changes as well. For instance, with swap_main, the focused window becomes the master but the cursor does not follow, rather remaining over its previous tile without focus. Similarly swap_next and swap_previous do not work as expected. Changing layouts via cycle_layout or flip_layout or any of the master/stack increasing/decreasing presents the same problem. Ideally, the cursor would never end up over an unfocused window with warp_pointer and focus-follows-mouse. With moving floating windows this becomes a slight issue, but I feel like floating layouts don't require the same strictness as tiled layouts in this way, because it maybe more likely that the user is continuously using the cursor with …

### #71 [closed] warp_pointer is not obeyed on window move  (1c)
When you move a window through the stack with swap_next, swap_prev, swap_main, etc., the pointer does not move with the window as would be expected (at least by this user) when warp_pointer = 1.

### #81 [closed] focus_urgent does not focus to urgent master windows  (0c)
If a window is master and it sets the urgent hint, the bar indicates that the urgent hint has been set; however, focus_urgent does not work either to bring the window's workspace to the current region or to focus the window when such workspace is then active in the then-current region.

### #87 [closed] quirk FOCUSPREV doesn't update border color  (8c)
When setting the FOCUSPREV quirk on a window, and the window is unmapped, the focus is given to the previous window but the border color isn't updated accordingly.

### #104 [closed] Fails to build on FreeBSD  (1c)
Fails to build on FreeBSD since commit 42bccafc30073c1428d63b93d5db8b749491c8d4 spectrwm.c:6377:2: error: type name requires a specifier or qualifier SIMPLEQ_ENTRY(event) entry; ^ spectrwm.c:6377:16: error: a parameter list without types is only allowed in a function definition SIMPLEQ_ENTRY(event) entry; ^ spectrwm.c:6377:22: error: expected ';' at end of declaration list SIMPLEQ_ENTRY(event) entry; ^ ; spectrwm.c:6380:34: error: expected function body after function declarator SIMPLEQ_HEAD(event_queue, event) events = SIMPLEQ_HEAD_INITIALIZER(events); ^ spectrwm.c:6389:12: warning: implicit declaration of function 'SIMPLEQ_FIRST' is invalid in C99 [-Wimplicit-function-declaration] if ((ep = SIMPLEQ_FIRST(&events))) { ^ spectrwm.c:6389:27: error: use of undeclared identifier 'events' if ((ep = SIMPLEQ_FIRST(&events))) { ^ spectrwm.c:6391:3: warning: implicit declaration of function …

