// ToDo: output of curve fitting funtion from lane rs -> coefficients have to be used here

pub fn required_angle() {
    /*1. because these are parallel lines, the coeeficients of the polynomils should be equal, and only the offsets(constants) should have some difference
    2. you can subtract the polynomials and get the distance between the 2 lines(width of the lane)
    3. now take the distance, devide it by 2, and replace it with the offset of any one of the polynomial
    4. now you got the polynomial which will pass from the center of the lane
    5. subtract the center from the image center to get the deviation
    6. based on the deviation (- or + and value), call the servo apis accordingly*/
}
