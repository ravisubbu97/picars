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


ruspy.main_init()
us_check()
