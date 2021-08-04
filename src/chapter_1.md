# Introduction

## Primary Structure
the hyperion (hyper-ion) framework uses the [ion shell](https://doc.redox-os.org/ion-manual/html/) to serve web requests dynamical in a similar manner to PHP with a few differences that are outlined below

### How Are Routes determined
like PHP routes are determined based on where html/css/js/ion files are placed on the file system, with a predefined root, such as /var/www/html in LAMP (linux, apache, mysql, PHP), however unlike LAMP, hyperion doesn't read files to determine what is and isn't PHP, instead it simply uses file extensions. This is done because it is easier to implement and solves some security vulnerabilities relating to embedding PHP code in a file that can be uploaded to a server, such as a JPEG file

