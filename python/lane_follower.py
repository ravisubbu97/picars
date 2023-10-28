import math
import time

import cv2
import numpy as np
from checks import create_video_capture, try_func
from lane_detector import LaneDetector

import ruspy

theta = 0
minLineLength = 5
maxLineGap = 10


def run_robot(secs=10):
    started = time.time()
    vid_cap = create_video_capture(640, 480, 30)
    motors = ruspy.motors_init(50, 100)
    motors.speed(100, 100)
    # motors.forward(100)
    # time.sleep(0.5)

    while (time.time() - started) < secs:
        print("VIDEO CAPTURE STARTED")
        ret, frame = vid_cap.read()
        if not ret:
            print("FRAME NOT CAPTURED")
            continue
        print("FRAME CAPTURED")
        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        blurred = cv2.GaussianBlur(gray, (5, 5), 0)
        edged = cv2.Canny(blurred, 85, 85)
        lines = cv2.HoughLinesP(edged, 1, np.pi / 180, 10, minLineLength, maxLineGap)

        if lines is None:
            print("NO LINES DETECTED")
        else:
            print("CALCULATE THETA")
            for x in range(0, len(lines)):
                for x1, y1, x2, y2 in lines[x]:
                    cv2.line(frame, (x1, y1), (x2, y2), (0, 255, 0), 2)
                    theta += math.atan2((y2 - y1), (x2 - x1))
                    print(theta)

            threshold = 6
            if theta > threshold:
                print("LEFT")
                motors.turn_left(100)
            if theta < -threshold:
                print("RIGHT")
                motors.turn_right(100)
            if abs(theta) < threshold:
                print("STRAIGHT")
                motors.forward(100)

            theta = 0

    print("STOPPING MOTORS")
    motors.stop()


def run_robot_with_nn(secs=10):
    started = time.time()
    vid_cap = create_video_capture(640, 480, 30)
    ld = LaneDetector(image_width=640, image_height=480)
    motors = ruspy.motors_init(50, 100)
    motors.speed(100, 100)
    # motors.forward(100)
    # time.sleep(0.5)

    # create black image to add left and right lanes
    lane_img = np.zeros((640, 480), 3, dtype=np.uint8)

    while (time.time() - started) < secs:
        print("VIDEO CAPTURE STARTED")
        ret, frame = vid_cap.read()
        if not ret:
            print("FRAME NOT CAPTURED")
            continue
        print("FRAME CAPTURED")
        _, _, left, right = ld(frame)
        lane_img[left > 0.5, :] = [255, 255, 255]
        lane_img[right > 0.5, :] = [255, 255, 255]
        gray = cv2.cvtColor(lane_img, cv2.COLOR_BGR2GRAY)
        blurred = cv2.GaussianBlur(gray, (5, 5), 0)
        edged = cv2.Canny(blurred, 85, 85)
        lines = cv2.HoughLinesP(edged, 1, np.pi / 180, 10, minLineLength, maxLineGap)

        if lines is None:
            print("NO LINES DETECTED")
        else:
            print("CALCULATE THETA")
            for x in range(0, len(lines)):
                for x1, y1, x2, y2 in lines[x]:
                    # cv2.line(frame, (x1, y1), (x2, y2), (0, 255, 0), 2)
                    theta += math.atan2((y2 - y1), (x2 - x1))
                    print(theta)

            threshold = 6
            if theta > threshold:
                print("LEFT")
                motors.turn_left(100)
            if theta < -threshold:
                print("RIGHT")
                motors.turn_right(100)
            if abs(theta) < threshold:
                print("STRAIGHT")
                motors.forward(100)

            theta = 0

    print("STOPPING MOTORS")
    motors.stop()


if __name__ == "__main__":
    ruspy.main_init()
    try_func(run_robot_with_nn)
    ruspy.reset_mcu()
