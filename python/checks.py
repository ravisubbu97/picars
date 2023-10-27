import time
import traceback

import cv2
import rustimport.import_hook  # noqa: F401
from lane_detector import LaneDetector
from traffic_light import detect

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
def motors_check():
    motors = ruspy.motors_init(50, 100)
    motors.forward(10)
    time.sleep(3)
    motors.turn_left(5)
    time.sleep(3)
    motors.turn_right(15)
    time.sleep(3)
    motors.backward(20)
    time.sleep(3)
    motors.stop()


# Servos example check
def servos_check():
    camera_servo_pin1, camera_servo_pin2, dir_servo_pin = ruspy.servos_init(
        [10, 20, 30]
    )
    camera_servo_pin1.angle(90)
    time.sleep(1)
    camera_servo_pin2.angle(90)
    time.sleep(1)
    dir_servo_pin.angle(90)
    time.sleep(1)


# Cameras example check
def create_video_capture(h=224, w=224, fps=10):
    vid_cap = cv2.VideoCapture(0, cv2.CAP_V4L2)
    vid_cap.set(cv2.CAP_PROP_FRAME_WIDTH, h)
    vid_cap.set(cv2.CAP_PROP_FRAME_HEIGHT, w)
    vid_cap.set(cv2.CAP_PROP_FPS, fps)

    return vid_cap


def run_preds_with_fps(vid_cap, ld, secs=10):
    started = time.time()
    last_logged = time.time()
    frame_count = 0
    frames_missed = 0

    while (time.time() - started) < secs:
        # read frame
        ret, cv_image = vid_cap.read()
        if not ret:
            frames_missed += 1
            print(f"[{frames_missed}]: failed to read frame")
            continue

        left_poly, right_poly, left, right = ld(cv_image)
        print(f"{left_poly = } {right_poly = }\n{left.shape = } {right.shape = }")
        # log model performance
        frame_count += 1
        now = time.time()
        if now - last_logged > 1:
            print(f"{frame_count / (now-last_logged)} fps")
            last_logged = now
            frame_count = 0


def run_preds(vid_cap, ld):
    # read frame
    ret, cv_image = vid_cap.read()
    if not ret:
        print("FRAME NOT CAPTURED")
        return None

    detect(cv_image)
    left_poly, right_poly, left, right = ld(cv_image)
    print(f"{left_poly = } {right_poly = }\n{left.shape = } {right.shape = }")


def cameras_check():
    vid_cap = create_video_capture()
    ld = LaneDetector(image_width=224, image_height=224)

    run_preds(vid_cap, ld)


def try_func(func):
    try:
        ruspy.main_init()
        func()
    except Exception as e:
        print(f"ERROR in {func.__name__}: {e}")
        traceback.print_exc()
    finally:
        print("FINAL RESET")
        ruspy.reset_mcu()


def checks():
    try_func(us_check)
    try_func(motors_check)
    try_func(servos_check)
    try_func(cameras_check)


if __name__ == "__main__":
    checks()
