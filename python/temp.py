import torch
import torch.nn.functional as F
from torchvision import transforms

import cv2
import time

def create_model(model_path="model.pth"):
    model = torch.load(model_path, map_location=torch.device('cpu'))
    model.eval()

    return model

def create_video_capture(h=224, w=224, fps=36):
    vid_cap = cv2.VideoCapture(0, cv2.CAP_V4L2)
    vid_cap.set(cv2.CAP_PROP_FRAME_WIDTH, h)
    vid_cap.set(cv2.CAP_PROP_FRAME_HEIGHT, w)
    vid_cap.set(cv2.CAP_PROP_FPS, fps)

    return vid_cap

def create_preprocessor():
    preprocess = transforms.Compose([
        transforms.ToTensor(),
        transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
    ])

    return preprocess

def run_preds(model, vid_cap, preprocess, secs=10):
    started = time.time()
    last_logged = time.time()
    frame_count = 0
    frames_missed = 0

    with torch.no_grad():
        while (time.time() - started) < secs:
            # read frame
            ret, image = vid_cap.read()
            if not ret:
                frames_missed += 1
                print(f"[{frames_missed}]: failed to read frame")
                continue

            # convert opencv output from BGR to RGB
            image = image[:, :, [2, 1, 0]]

            # preprocess
            input_tensor = preprocess(image)

            # create a mini-batch as expected by the model
            input_batch = input_tensor.unsqueeze(0)

            # run model
            out = F.softmax(model(input_batch), dim=1)
            background, left, right = out[0,0,:,:], out[0,1,:,:], out[0,2,:,:]

            # log model performance
            frame_count += 1
            now = time.time()
            if now - last_logged > 1:
                print(f"{frame_count / (now-last_logged)} fps")
                last_logged = now
                frame_count = 0

model = create_model()
vid_cap = create_video_capture()
preprocess = create_preprocessor()
run_preds(model, vid_cap, preprocess, secs=0.001)
