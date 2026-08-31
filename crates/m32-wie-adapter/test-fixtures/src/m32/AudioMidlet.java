package m32;

import java.io.ByteArrayInputStream;
import javax.microedition.media.Manager;
import javax.microedition.media.Player;
import javax.microedition.midlet.MIDlet;

public final class AudioMidlet extends MIDlet {
    public AudioMidlet() { super(); }

    protected void startApp() {
        try {
            Player player = Manager.createPlayer(
                new ByteArrayInputStream(new byte[0]),
                "application/vnd.smaf"
            );
            player.start();
            System.out.print("M32_AUDIO_PLAY_SENT;");
            System.out.flush();
            player.stop();
            System.out.print("M32_AUDIO_STOP_SENT;");
            System.out.flush();
        } catch (Exception error) {
            System.out.print("M32_AUDIO_FAILURE;");
            System.out.flush();
        }
    }

    protected void pauseApp() {}
    protected void destroyApp(boolean unconditional) {}
}
