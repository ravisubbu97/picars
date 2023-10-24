from lane_detector import LaneDetector

import cv2
import time

def create_video_capture(h=224, w=224, fps=10):
    vid_cap = cv2.VideoCapture(0, cv2.CAP_V4L2)
    vid_cap.set(cv2.CAP_PROP_FRAME_WIDTH, h)
    vid_cap.set(cv2.CAP_PROP_FRAME_HEIGHT, w)
    vid_cap.set(cv2.CAP_PROP_FPS, fps)

    return vid_cap

def run_preds(vid_cap, ld, secs=10):
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

vid_cap = create_video_capture()
ld = LaneDetector(image_width=224, image_height=224)

run_preds(vid_cap, ld)
