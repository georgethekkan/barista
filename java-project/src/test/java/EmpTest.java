import java.beans.Transient;
import java.lang.AssertionError;
import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class EmpTest {

    @Test
    public void test_emp() {
        var emp = new Emp("John", 30);
        assertEquals("John", emp.name());
        assertEquals(31, emp.age());
    }

    @Test
    public void test_emp1() {
        var emp = new Emp("John1", 30);
        assertEquals("John", emp.name());
        assertEquals(31, emp.age());

    }
}
