package m32;

import java.io.ByteArrayInputStream;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.media.Manager;
import javax.microedition.media.Player;
import javax.microedition.midlet.MIDlet;
import javax.microedition.rms.RecordStore;

public final class FirstPlayableMidlet extends MIDlet {
    private FirstPlayableCanvas canvas;

    public FirstPlayableMidlet() {
        super();
    }

    protected void startApp() {
        int counter = loadCounter();
        canvas = new FirstPlayableCanvas(this, counter);
        Display.getDisplay(this).setCurrent(canvas);
        System.out.print("M32_FP_RUNNING:");
        System.out.print(counter);
        System.out.print(";");
        System.out.flush();
    }

    void saveCounter(int counter) {
        RecordStore store = null;
        try {
            store = RecordStore.openRecordStore("m32-first-playable", true);
            byte[] data = new byte[] { (byte) counter };
            if (store.getNumRecords() == 0) {
                store.addRecord(data, 0, data.length);
            } else {
                store.setRecord(1, data, 0, data.length);
            }
            System.out.print("M32_FP_SAVED:");
            System.out.print(counter);
            System.out.print(";");
            System.out.flush();
        } catch (Exception error) {
            System.out.print("M32_FP_SAVE_FAILURE;");
            System.out.flush();
        } finally {
            if (store != null) {
                try {
                    store.closeRecordStore();
                } catch (Exception ignored) {
                }
            }
        }
    }

    void emitAudio(int counter) {
        try {
            Player player = Manager.createPlayer(
                new ByteArrayInputStream(new byte[0]),
                "application/vnd.smaf"
            );
            player.start();
            System.out.print("M32_FP_AUDIO:");
            System.out.print(counter);
            System.out.print(";");
            System.out.flush();
            player.stop();
        } catch (Exception error) {
            System.out.print("M32_FP_AUDIO_FAILURE;");
            System.out.flush();
        }
    }

    private int loadCounter() {
        RecordStore store = null;
        try {
            store = RecordStore.openRecordStore("m32-first-playable", true);
            if (store.getNumRecords() == 0) {
                return 0;
            }
            byte[] data = store.getRecord(1);
            if (data == null || data.length == 0) {
                return 0;
            }
            return data[0] & 0xFF;
        } catch (Exception error) {
            System.out.print("M32_FP_LOAD_FAILURE;");
            System.out.flush();
            return 0;
        } finally {
            if (store != null) {
                try {
                    store.closeRecordStore();
                } catch (Exception ignored) {
                }
            }
        }
    }

    protected void pauseApp() {
    }

    protected void destroyApp(boolean unconditional) {
    }

    private static final class FirstPlayableCanvas extends Canvas {
        private final FirstPlayableMidlet midlet;
        private int counter;

        FirstPlayableCanvas(FirstPlayableMidlet midlet, int counter) {
            super();
            this.midlet = midlet;
            this.counter = counter;
        }

        protected void paint(Graphics graphics) {
            graphics.setColor(0x0E1114);
            graphics.fillRect(0, 0, 176, 220);

            graphics.setColor(0xD14A36);
            graphics.fillRect(8, 8, 24, 8);

            graphics.setColor(0x5C9B76);
            int x = 16 + ((counter % 8) * 18);
            graphics.fillRect(x, 80, 16, 16);
            graphics.fillRect(16, 120, 8 + ((counter % 16) * 8), 8);
        }

        protected void keyPressed(int keyCode) {
            counter = (counter + 1) & 0xFF;
            midlet.saveCounter(counter);
            midlet.emitAudio(counter);
            repaint();
            System.out.print("M32_FP_INPUT:");
            System.out.print(counter);
            System.out.print(";");
            System.out.flush();
        }
    }
}
