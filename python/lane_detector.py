from camera_geometry import CameraGeometry

import numpy as np
import torch
import torch.nn.functional as F
from torchvision import transforms

class LaneDetector():
    def __init__(self, model_path="model.pth", *args, **kwargs):
        self.cam_geom = CameraGeometry(*args, **kwargs)
        self.cut_v, self.grid = self.cam_geom.precompute_grid()
        self.model = torch.load(model_path, map_location=torch.device('cpu'))
        self.model.eval()

    def create_preprocessor(self, cv_image):
        preprocess = transforms.Compose([
            transforms.ToTensor(),
            transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
        ])
        # convert opencv output from BGR to RGB
        image = cv_image[:, :, [2, 1, 0]]
        input_tensor = preprocess(image)
        input_batch = input_tensor.unsqueeze(0)

        return input_batch

    def predict(self, input_batch):
        with torch.no_grad():
            out = F.softmax(self.model(input_batch), dim=1)
            background, left, right = out[0,0,:,:], out[0,1,:,:], out[0,2,:,:]

        return background, left, right

    def fit_poly(self, probs, prob_thresh=0.3):
        probs_flat = np.ravel(probs[self.cut_v:, :])
        mask = probs_flat > prob_thresh
        if mask.sum() > 0:
            coeffs = np.polyfit(self.grid[:,0][mask], self.grid[:,1][mask], deg=3, w=probs_flat[mask])
        else:
            coeffs = np.array([0.,0.,0.,0.])

        return np.poly1d(coeffs)

    def get_fit_and_probs(self, cv_image):
        input_batch = self.create_preprocessor(cv_image)
        _, left, right = self.predict(input_batch)
        left_poly = self.fit_poly(left)
        right_poly = self.fit_poly(right)

        return left_poly, right_poly, left, right

    def __call__(self, cv_image):
        left_poly, right_poly, left, right = self.get_fit_and_probs(cv_image)

        return left_poly, right_poly, left, right
