package javax.microedition.rms;
public final class RecordStore {
    private RecordStore() {}
    public static RecordStore openRecordStore(String name, boolean create) { return null; }
    public int getNumRecords() { return 0; }
    public int addRecord(byte[] data, int offset, int length) { return 0; }
    public byte[] getRecord(int id) { return null; }
    public void setRecord(int id, byte[] data, int offset, int length) {}
    public void closeRecordStore() {}
}
