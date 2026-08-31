package m32;

import javax.microedition.midlet.MIDlet;
import javax.microedition.rms.RecordStore;

public final class RmsPersistenceMidlet extends MIDlet {
    public RmsPersistenceMidlet() { super(); }

    protected void startApp() {
        try {
            byte[] expected = new byte[] {
                0x4D, 0x33, 0x32, 0x2D, 0x52, 0x4D, 0x53, 0x31
            };
            RecordStore store = RecordStore.openRecordStore("m32-rms", true);

            if (store.getNumRecords() == 0) {
                int id = store.addRecord(expected, 0, expected.length);
                if (id == 1) {
                    System.out.print("M32_RMS_SAVED;");
                } else {
                    System.out.print("M32_RMS_BAD_ID;");
                }
            } else {
                byte[] loaded = store.getRecord(1);
                if (sameBytes(expected, loaded)) {
                    System.out.print("M32_RMS_LOADED_OK;");
                } else {
                    System.out.print("M32_RMS_LOADED_BAD;");
                }
            }

            store.closeRecordStore();
            System.out.flush();
        } catch (Exception error) {
            System.out.print("M32_RMS_FAILURE;");
            System.out.flush();
        }
    }

    private static boolean sameBytes(byte[] expected, byte[] actual) {
        if (actual == null || expected.length != actual.length) {
            return false;
        }

        for (int i = 0; i < expected.length; i++) {
            if (expected[i] != actual[i]) {
                return false;
            }
        }

        return true;
    }

    protected void pauseApp() {}
    protected void destroyApp(boolean unconditional) {}
}
