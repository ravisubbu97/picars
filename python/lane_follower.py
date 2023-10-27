import math
import time

import cv2
import numpy as np
from checks import create_video_capture, try_func

import ruspy

theta = 0
minLineLength = 5
maxLineGap = 10


def run_robot(secs=10):
    started = time.time()
    print("VIDEO CAPTURE STARTED")
    vid_cap = create_video_capture(640, 480, 30)
    motors = ruspy.motors_init(50, 100)
    motors.speed(10, 10)
    motors.forward(10)
    time.sleep(1)

    while (time.time() - started) < secs:
        for ret, frame in vid_cap.read():
            if not ret:
                continue
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            blurred = cv2.GaussianBlur(gray, (5, 5), 0)
            edged = cv2.Canny(blurred, 85, 85)
            lines = cv2.HoughLinesP(
                edged, 1, np.pi / 180, 10, minLineLength, maxLineGap
            )

            if lines is None:
                print("NO LINES DETECTED")
            else:
                for x in range(0, len(lines)):
                    for x1, y1, x2, y2 in lines[x]:
                        cv2.line(frame, (x1, y1), (x2, y2), (0, 255, 0), 2)
                        theta += math.atan2((y2 - y1), (x2 - x1))
                        print(theta)

                threshold = 6
                if theta > threshold:
                    print("LEFT")
                    motors.turn_left(5)
                if theta < -threshold:
                    print("RIGHT")
                    motors.turn_right(15)
                if abs(theta) < threshold:
                    print("STRAIGHT")
                    motors.forward(10)

            theta = 0

    motors.stop()


if __name__ == "__main__":
    ruspy.main_init()
    try_func(run_robot)
    ruspy.reset_mcu()
