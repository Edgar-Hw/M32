// M32 synthetic J2ME audio source fixture.
// License: MIT, M32 contributors.

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Form;
import javax.microedition.media.Manager;
import javax.microedition.media.Player;

public final class AudioMidlet extends MIDlet {
    private final Form form = new Form("M32 Audio Fixture");
    private Player player;

    protected void startApp() {
        Display.getDisplay(this).setCurrent(form);
        try {
            // Deliberately uses a generated tone sequence, not copyrighted music.
            byte[] sequence = new byte[] {
                0x01, 0x00,
                0x02, 0x1E,
                0x03, 0x40,
                60, 8,
                64, 8,
                67, 8
            };
            player = Manager.createPlayer(
                new java.io.ByteArrayInputStream(sequence),
                "audio/x-tone-seq"
            );
            player.start();
            form.append("AUDIO-STARTED");
        } catch (Exception e) {
            form.append("AUDIO-ERROR");
        }
    }

    protected void pauseApp() {
    }

    protected void destroyApp(boolean unconditional) {
        if (player != null) {
            player.close();
        }
    }
}
