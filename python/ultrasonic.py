import time

import rustimport.import_hook  # noqa: F401

import dust


# Ultrasonic example check
def us_check():
    for _ in range(5):
        us = dust.ultrasonic_init()
        distance = us.read()
        print(f"Distance: {distance} cm")
        # Sleep for 60 milliseconds (as per DATASHEET) --> FIX ME: consider ultrasonic.read() timing into account
        time.sleep(0.06)


dust.main_init()
us_check()
