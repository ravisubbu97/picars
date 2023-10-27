import time

import rustimport.import_hook  # noqa: F401

import ruspy


# Ultrasonic example check
def us_check():
    us = ruspy.ultrasonic_init()
    for _ in range(5):
        distance = us.read()
        print(f"Distance: {distance} cm")
        # Sleep for 60 milliseconds (as per DATASHEET) --> FIX ME: consider ultrasonic.read() timing into account
        time.sleep(0.06)


# Motors example check
def motors_chcek():
    motors = ruspy.motors_init()
    motors.forward(10)
    motors.turn_left(5)
    motors.turn_right(15)
    motors.backward(20)
    motors.stop()


# Servos example check
def servos_check():
    camera_servo_pin1, camera_servo_pin2, dir_servo_pin = ruspy.servos_init(
        [10, 20, 30]
    )
    camera_servo_pin1.angle(90)
    camera_servo_pin2.angle(90)
    dir_servo_pin.angle(90)


ruspy.main_init()
us_check()
motors_chcek()
servos_check()
