import cv2
import numpy as np


def detect(cv_img):
    font = cv2.FONT_HERSHEY_SIMPLEX
    cimg = cv_img
    hsv = cv2.cvtColor(cv_img, cv2.COLOR_BGR2HSV)

    # color range
    lower_red1 = np.array([0, 100, 100])
    upper_red1 = np.array([10, 255, 255])
    lower_red2 = np.array([160, 100, 100])
    upper_red2 = np.array([180, 255, 255])
    lower_green = np.array([40, 50, 50])
    upper_green = np.array([90, 255, 255])
    mask1 = cv2.inRange(hsv, lower_red1, upper_red1)
    mask2 = cv2.inRange(hsv, lower_red2, upper_red2)
    maskg = cv2.inRange(hsv, lower_green, upper_green)
    maskr = cv2.add(mask1, mask2)

    size = cv_img.shape

    # hough circle detect
    r_circles = cv2.HoughCircles(
        maskr,
        cv2.HOUGH_GRADIENT,
        1,
        80,
        param1=50,
        param2=10,
        minRadius=0,
        maxRadius=30,
    )

    g_circles = cv2.HoughCircles(
        maskg,
        cv2.HOUGH_GRADIENT,
        1,
        60,
        param1=50,
        param2=10,
        minRadius=0,
        maxRadius=30,
    )

    # traffic light detect
    r = 5
    bound = 4.0 / 10
    if r_circles is not None:
        r_circles = np.uint16(np.around(r_circles))

        for i in r_circles[0, :]:
            if i[0] > size[1] or i[1] > size[0] or i[1] > size[0] * bound:
                continue

            h, s = 0.0, 0.0
            for m in range(-r, r):
                for n in range(-r, r):
                    if (i[1] + m) >= size[0] or (i[0] + n) >= size[1]:
                        continue
                    h += maskr[i[1] + m, i[0] + n]
                    s += 1
            if h / s > 50:
                cv2.circle(cimg, (i[0], i[1]), i[2] + 10, (0, 255, 0), 2)
                cv2.circle(maskr, (i[0], i[1]), i[2] + 30, (255, 255, 255), 2)
                cv2.putText(
                    cimg, "RED", (i[0], i[1]), font, 1, (255, 0, 0), 2, cv2.LINE_AA
                )

    if g_circles is not None:
        g_circles = np.uint16(np.around(g_circles))

        for i in g_circles[0, :]:
            if i[0] > size[1] or i[1] > size[0] or i[1] > size[0] * bound:
                continue

            h, s = 0.0, 0.0
            for m in range(-r, r):
                for n in range(-r, r):
                    if (i[1] + m) >= size[0] or (i[0] + n) >= size[1]:
                        continue
                    h += maskg[i[1] + m, i[0] + n]
                    s += 1
            if h / s > 100:
                cv2.circle(cimg, (i[0], i[1]), i[2] + 10, (0, 255, 0), 2)
                cv2.circle(maskg, (i[0], i[1]), i[2] + 30, (255, 255, 255), 2)
                cv2.putText(
                    cimg, "GREEN", (i[0], i[1]), font, 1, (255, 0, 0), 2, cv2.LINE_AA
                )

    # cv2.imshow("detected results", cimg)
    cv2.imwrite("circles.jpg", cimg)

    # cv2.imshow('maskr', maskr)
    # cv2.imshow('maskg', maskg)
    # cv2.imshow('masky', masky)

    # cv2.waitKey(0)
    # cv2.destroyAllWindows()


if __name__ == "__main__":
    cv_img = cv2.imread("red.jpg")
    detect(cv_img)
