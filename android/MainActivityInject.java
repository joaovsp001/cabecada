// Extra Java code for the Android build. cargo-quad-apk copies each block below into
// the matching "//% NAME" spot of miniquad's MainActivity.java (see quad.toml).
// Blocks end with "//% END". Keep everything inside the blocks.
// The cargo-quad-apk in the build image (2024) only knows these blocks: IMPORTS,
// MAIN_ACTIVITY_BODY, MAIN_ACTIVITY_ON_CREATE, MAIN_ACTIVITY_ON_RESUME,
// MAIN_ACTIVITY_ON_PAUSE and MAIN_ACTIVITY_ON_ACTIVITY_RESULT; any other one stops the build.
// (With the system bars hidden, miniquad's ResizingLayout gets zero insets, so the game
// already fills the whole screen.)

//% IMPORTS
import android.view.WindowInsetsController;
import android.net.wifi.WifiManager;
//% END

//% MAIN_ACTIVITY_BODY
    // Lets the game find matches on the local network: many phones drop broadcast
    // packets on Wi-Fi unless an app holds a multicast lock.
    private WifiManager.MulticastLock cabecadaMulticastLock;

    // True full screen: hide the status bar and the navigation bar (swipe to show them).
    private void cabecadaHideSystemBars() {
        try {
            if (Build.VERSION.SDK_INT >= 30) {
                WindowInsetsController controller = getWindow().getInsetsController();
                if (controller != null) {
                    controller.hide(WindowInsets.Type.systemBars());
                    controller.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                }
            } else {
                getWindow().getDecorView().setSystemUiVisibility(
                    View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                    | View.SYSTEM_UI_FLAG_FULLSCREEN
                    | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                    | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                    | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                    | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION);
            }
        } catch (Exception e) {
            Log.w("Cabecada", "could not hide the system bars: " + e);
        }
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            cabecadaHideSystemBars();
        }
    }

    // The back button always goes to the game (it pauses a match, leaves the lobbies and
    // asks before closing). Without this, when the game view is not focused Android
    // handles back itself and closes the screen.
    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getKeyCode() == KeyEvent.KEYCODE_BACK) {
            if (event.getAction() == KeyEvent.ACTION_DOWN && event.getRepeatCount() == 0) {
                QuadNative.surfaceOnKeyDown(KeyEvent.KEYCODE_BACK);
            } else if (event.getAction() == KeyEvent.ACTION_UP) {
                QuadNative.surfaceOnKeyUp(KeyEvent.KEYCODE_BACK);
            }
            return true;
        }
        return super.dispatchKeyEvent(event);
    }
//% END

//% MAIN_ACTIVITY_ON_CREATE
        cabecadaHideSystemBars();
        // The game engine can only start once per process: starting it again in the same
        // process crashes. So when Android destroys this screen, end the process too.
        // If Android is only re-creating the screen (a setting changed: rotation while
        // opening, dark mode, display size...), first ask for a fresh start of the game,
        // which then runs in a new, clean process.
        getApplication().registerActivityLifecycleCallbacks(new android.app.Application.ActivityLifecycleCallbacks() {
            public void onActivityCreated(Activity a, Bundle b) {}
            public void onActivityStarted(Activity a) {}
            public void onActivityResumed(Activity a) {}
            public void onActivityPaused(Activity a) {}
            public void onActivityStopped(Activity a) {}
            public void onActivitySaveInstanceState(Activity a, Bundle b) {}
            public void onActivityDestroyed(Activity a) {
                if (a == MainActivity.this) {
                    boolean recreating = a.isChangingConfigurations();
                    Log.i("Cabecada", "screen destroyed (finishing=" + a.isFinishing()
                        + ", changed config=0x" + Integer.toHexString(a.getChangingConfigurations())
                        + "): ending the process" + (recreating ? " and starting the game again" : ""));
                    if (recreating) {
                        try {
                            Intent again = new Intent(a, MainActivity.class);
                            again.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK);
                            a.startActivity(again);
                        } catch (Exception e) {
                            Log.w("Cabecada", "could not start the game again: " + e);
                        }
                    }
                    android.os.Process.killProcess(android.os.Process.myPid());
                }
            }
        });
//% END

//% MAIN_ACTIVITY_ON_RESUME
        cabecadaHideSystemBars();
        try {
            if (cabecadaMulticastLock == null) {
                WifiManager wifi = (WifiManager) getApplicationContext().getSystemService(Context.WIFI_SERVICE);
                if (wifi != null) {
                    cabecadaMulticastLock = wifi.createMulticastLock("cabecada");
                    cabecadaMulticastLock.setReferenceCounted(false);
                }
            }
            if (cabecadaMulticastLock != null) {
                cabecadaMulticastLock.acquire();
            }
        } catch (Exception e) {
            Log.w("Cabecada", "no multicast lock: " + e);
        }
//% END

//% MAIN_ACTIVITY_ON_PAUSE
        try {
            if (cabecadaMulticastLock != null && cabecadaMulticastLock.isHeld()) {
                cabecadaMulticastLock.release();
            }
        } catch (Exception e) {
            Log.w("Cabecada", "could not release the multicast lock: " + e);
        }
//% END
