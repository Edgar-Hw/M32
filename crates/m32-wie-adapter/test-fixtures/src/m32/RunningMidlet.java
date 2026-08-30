package m32;

import javax.microedition.midlet.MIDlet;

public final class RunningMidlet extends MIDlet {
    public RunningMidlet() {
        super();
    }

    protected void startApp() {
        System.out.println("M32_FIRST_FRAME_BOOT_OK");
        System.out.flush();
    }
}
