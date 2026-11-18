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
//% END

//% MAIN_ACTIVITY_ON_CREATE
        cabecadaHideSystemBars();
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
